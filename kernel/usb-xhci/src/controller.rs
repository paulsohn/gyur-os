extern crate alloc;

use core::alloc::Allocator;
use core::cell::RefCell;
use core::marker::PhantomData;
use alloc::alloc::Global;
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::endpoint::{EndpointAddress, EndpointType, EndpointConfig};
use crate::setup::SetupRequest;

use crate::ring::{CommandRing, EventRing};

use crate::device::Device;
use crate::bus::{USBBus, XHCIBus};
use crate::class::{USBClass, SupportedClassListeners};

use volatile::VolatilePtr;
// use volatile::map_field;
use volatile_field::Structural;

use xhci::registers::{
    Registers,
    operational::PortStatusAndControlRegister,
    doorbell::Doorbell,
    extended_capabilities::ExtendedCapabilities,
};
use xhci::ring::trb::{transfer, event, command};
use xhci::context;

pub const MAX_DEVICE_SLOTS: usize = 8;

pub struct DeviceEntry<B, L, A = Global>
where
    B: USBBus,
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    pub(crate) device: RefCell<Device<B, L, A>>,
    pub(crate) bus: B,
    pub(crate) class_drivers: RefCell<Vec<Box<dyn USBClass, A>, A>>,
    pub(crate) ep_configs: RefCell<Vec<EndpointConfig, A>>,
}
pub type XHCIDeviceEntry<'r, L, A = Global> = DeviceEntry<XHCIBus<'r, L, A>, L, A>;

impl<'r, L, A> XHCIDeviceEntry<'r, L, A>
where
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    pub fn new(
        db: VolatilePtr<'r, Doorbell>, 
        use_64byte: bool,
        allocator: A
    ) -> Self {
        Self {
            device: RefCell::new(Device::new(allocator.clone())),
            bus: XHCIBus::new(db, use_64byte, allocator.clone()),
            class_drivers: RefCell::new(Vec::new_in(allocator.clone())),
            ep_configs: RefCell::new(Vec::new_in(allocator)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C, align(64))]
pub struct DeviceContextBaseAddressArray {
    scratchpad: *mut *const core::mem::MaybeUninit<u8>,
    ctx_base_ptrs: [*mut context::Output32Byte; MAX_DEVICE_SLOTS],
}

impl DeviceContextBaseAddressArray {
    pub fn new() -> Self {
        // Self {
        //     scratchpad: core::ptr::null_mut(),
        //     ctx_base_ptrs: [core::ptr::null_mut(); MAX_DEVICE_SLOTS],
        // }
        unsafe {
            core::mem::MaybeUninit::<Self>::zeroed().assume_init()
        }
    }
}

pub struct XHCIDeviceManager<'r, L, A = Global>
where
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    pub(crate) dcbaa: Box<DeviceContextBaseAddressArray, A>,

    entries: [Option<Box<XHCIDeviceEntry<'r, L, A>, A>>; MAX_DEVICE_SLOTS],

    _listeners: PhantomData<L>,
    allocator: A,
}

impl<'r, L, A> XHCIDeviceManager<'r, L, A>
where
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    pub fn new(allocator: A) -> Self {
        Self {
            dcbaa: Box::new_in(DeviceContextBaseAddressArray::new(), allocator.clone()),
            entries: core::array::from_fn(|_| None),
            _listeners: PhantomData,
            allocator,
        }
    }

    pub fn alloc_entry(&mut self, slot_id: usize, use_64byte: bool, db: VolatilePtr<'r, Doorbell>) -> &Box<XHCIDeviceEntry<'r, L, A>, A> {
        assert!(slot_id <= MAX_DEVICE_SLOTS);

        if self.entries[slot_id - 1].is_some() {
            panic!("Device for slot {slot_id} already allocated.");
            // return;
        }

        let new_entry = Box::new_in(
            XHCIDeviceEntry::new(db, use_64byte, self.allocator.clone()),
            self.allocator.clone()
        );

        // update DCBAA
        self.dcbaa.ctx_base_ptrs[slot_id - 1] = new_entry.bus.output_ctx_ptr();

        self.entries[slot_id - 1] = Some(new_entry);
        self.entries[slot_id - 1].as_ref().unwrap()
    }

    pub(crate) fn set_scratchpad_buffer_array(&mut self, sp_ptr: *mut *const core::mem::MaybeUninit<u8>) {
        self.dcbaa.scratchpad = sp_ptr;
    }

    pub fn entry_at(&self, slot_id: usize) -> Option<&Box<XHCIDeviceEntry<'r, L, A>, A>> {
        self.entries[slot_id - 1].as_ref()
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PortConfigPhase {
    NotConnected,
    WaitingAddressed,
    ResettingPort,
    EnablingSlot,
    AddressingDevice,
    InitializingDevice,
    ConfiguringEndpoints,
    Configured
}

/// A xCHI controller.
pub struct Controller<'r, L, A = Global>
where
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    regs: Registers<'r>,

    dev_mgr: XHCIDeviceManager<'r, L, A>,
    cmd_ring: RefCell<CommandRing<A>>,
    ev_ring: EventRing<'r, A>,

    // Below are controller global variable in MikanOS.

    port_cfg_phase: [PortConfigPhase; 256], // port phase of port_id `i` is `port_cfg_phase[i - 1]`.
    addressing_port: usize, // if 0, no addressing port. It's more like Option<NonzeroUsize>>.

    use_64byte_context: bool,

    // allocator: A,
}

impl<'r, L, A> Controller<'r, L, A>
where
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    pub fn new(mmio_base: u64, allocator: A) -> Self {
        let mut regs = unsafe {
            Registers::new(mmio_base as usize)
        };

        let extcap_list = regs.extended_capabilities();

        let cap = &regs.capability;
        // let hccparams1 = map_field!(cap.hccparams1).read();
        // let hcsparams2 = map_field!(cap.hcsparams2).read();
        let hccparams1 = cap.fields().hccparams1().read();
        let hcsparams2 = cap.fields().hcsparams2().read();

        // Request Host Controller ownership
        {
            let opt_ptr_legsup = extcap_list.into_iter().find_map(|ext| {
                // ext.ok().map(|ext| {
                //     if let ExtendedCapability::UsbLegacySupport(ext_cap_usb) = ext {
                //         Some(ext_cap_usb.usblegsup)
                //     } else {
                //         None
                //     }
                // }).flatten()

                if let Ok(ExtendedCapabilities::UsbLegacySupport(ext_cap_usb)) = ext {
                    Some(ext_cap_usb.ptr)
                } else {
                    None
                }
            });

            if let Some(ptr_legsup) = opt_ptr_legsup {
                let mut legsup = ptr_legsup.read();
                if !legsup.cap_header.hc_os_owned_semaphore() {
                    legsup.cap_header.set_hc_os_owned_semaphore();
                    ptr_legsup.write(legsup);

                    // wait until os gets controller ownership.
                    while {
                        let legsup = ptr_legsup.read();
                        !legsup.cap_header.hc_os_owned_semaphore() || legsup.cap_header.hc_bios_owned_semaphore()
                    } {}
                }
            }
        }

        let op = &mut regs.operational;
        // let op_usbcmd = map_field!(op.usbcmd);
        // let op_usbsts = map_field!(op.usbsts);
        let op_usbcmd = op.fields().usbcmd();
        let op_usbsts = op.fields().usbsts();

        // disable interrupt for controller and stop
        {
            let hc_halted = op_usbsts.read().hc_halted();

            op_usbcmd.update(|mut usbcmd| {
                usbcmd.clear_interrupter_enable()
                    .clear_host_system_error_enable()
                    .clear_enable_wrap_event();
                if hc_halted {
                    usbcmd.clear_run_stop(); // stop
                }
                usbcmd
            });

            // wait until hc has halted
            while !op_usbsts.read().hc_halted() {}
        }

        // todo: read page size

        // Reset controller.
        {
            op_usbcmd.update(|mut usbcmd| {
                *usbcmd.set_host_controller_reset()
            });

            // wait until `hc_reset` bit has been consumed.
            while op_usbcmd.read().host_controller_reset() {}

            // wait until controller is ready.
            while op_usbsts.read().controller_not_ready() {}
        }

        // Set max device slots.
        {
            op.fields().config().update(|mut config| {
                let slots = MAX_DEVICE_SLOTS as u8;
                *config.set_max_device_slots_enabled(slots)
            });
        }

        // init device manager.
        let mut dev_mgr = XHCIDeviceManager::new(allocator.clone());

        // Allocate scratchpad buffer arrays.
        {
            let max_sp_buffers = hcsparams2.max_scratchpad_buffers();

            if max_sp_buffers > 0 {
                let mut sp_buffers: Vec<*const core::mem::MaybeUninit<u8>, A> = Vec::new_in(allocator.clone());
                sp_buffers.reserve(max_sp_buffers as usize);

                for _ in 0..max_sp_buffers {
                    // Allocate a page.

                    const PAGE_BYTES: usize = 4096;

                    let buf = Box::new_uninit_slice_in(PAGE_BYTES, allocator.clone());
                    sp_buffers.push( Box::into_raw(buf).as_mut_ptr() );
                }

                // pass `sp_buffers` as a raw ptr so that it won't be dropped
                dev_mgr.set_scratchpad_buffer_array(sp_buffers.into_raw_parts_with_alloc().0);
            }
        }

        // set DCBAA Pointer
        {
            let dcbaa_ptr = &*dev_mgr.dcbaa as *const _ as usize as u64;
            op.fields().dcbaap().update(|mut dcbaap| {
                *dcbaap.set(dcbaa_ptr)
            });
        }

        // initialize Command Ring.
        let cmd_ring = {
            let cmd_ring = CommandRing::new(32, allocator.clone());

            let buf_ptr = unsafe { cmd_ring.get_buf_ptr(0) };

            // register this ring
            op.fields().crcr().update(|mut crcr| {
                *crcr.set_ring_cycle_state()
                    // .clear_command_stop()
                    // .clear_command_abort()
                    .set_command_ring_pointer(buf_ptr as usize as u64)
            });

            RefCell::new(cmd_ring)
        };

        // initialize Event Ring and its primary interrupter (interrupter 0)
        let ev_ring = {
            // The primary interrupter.
            let interrupter = regs.interrupter_register_set_array.index(0);

            // enable interrupt for primary interrupter
            interrupter.fields().iman().update(|mut iman| {
                *iman.clear_interrupt_pending() // RW1C, this writes 1 to clear
                    .set_interrupt_enable()
            });

            // enable interrupt for controller
            op_usbcmd.update(|mut usbcmd| {
                *usbcmd.set_interrupter_enable()
            });

            EventRing::new(interrupter, 32, allocator.clone())
            // the event ring is already registered on construction
        };

        Self {
            regs,
            dev_mgr,
            cmd_ring,
            ev_ring,

            // port_cfg_phase: core::array::from_fn(|_| PortConfigPhase::NotConnected),
            port_cfg_phase: [PortConfigPhase::NotConnected; 256],
            addressing_port: 0,

            use_64byte_context: hccparams1.context_size(),

            // allocator
        }
    }

    /// run the controller.
    pub fn run(&mut self) {
        let op = &mut self.regs.operational;

        // set run-stop bit
        {
            op.fields().usbcmd().update(|mut usbcmd| {
                *usbcmd.set_run_stop()
            });
            let _refresh = op.fields().usbcmd().read();

            // wait until hc is running
            while op.fields().usbsts().read().hc_halted() {}
        }
    }

    pub fn process_events(&mut self) {
        if let Some(block) = self.ev_ring.pop() {
            if let Ok(psc) = event::PortStatusChange::try_from(block) {
                self.on_port_status_change(psc.port_id() as usize);
            } else if let Ok(te) = event::TransferEvent::try_from(block) {
                self.on_transfer(te);
            } else if let Ok(cc) = event::CommandCompletion::try_from(block) {
                self.on_cmd_complete(cc);
            } else {
                unimplemented!("Unsupported Event TRB.");
            }
        }
    }

    pub fn reconfigure_port(&mut self) {
        for i in 1..=self.num_ports() {
            if self.port_is_connected(i) && self.port_cfg_phase[i - 1] == PortConfigPhase::NotConnected {
                self.reset_port(i);
            }
        }
    }
}

// Port basic functions.
impl<L, A> Controller<'_, L, A>
where
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    /// The port numbers, which is also the maximum valid port id.
    /// 
    /// i.e. the valid port numbers are `1..=num_ports`.
    fn num_ports(&self) -> usize {
        self.regs.port_register_set_array.len()
    }

    /// Returns `i`th port status and control register, where `i` is 1-indexed.
    /// 
    /// Panics when `i == 0`.
    fn portsc_at(&self, i: usize) -> PortStatusAndControlRegister {
        assert!(i > 0);
        self.regs.port_register_set_array.index(i - 1)
            .fields().portsc()
            .read()
    }

    fn port_is_connected(&self, i: usize) -> bool {
        self.portsc_at(i).current_connect_status()
    }

    fn port_is_enabled(&self, i: usize) -> bool {
        self.portsc_at(i).port_enabled_disabled()
    }

    fn port_is_port_reset_changed(&self, i: usize) -> bool {
        self.portsc_at(i).port_reset_change()
    }

    fn port_speed(&self, i: usize) -> u8 {
        self.portsc_at(i).port_speed()
    }

    /// Modifies `i`th port status and control register with the given function, where `i` is 1-indexed.
    /// 
    /// RW1C bits are protected on this function.
    /// 
    /// Panics when `i == 0`.
    fn protected_update_portsc_at<F>(&mut self, i: usize, f: F)
    where
        F: FnOnce(&mut PortStatusAndControlRegister)
    {
        fn portsc_protect(portsc: &mut PortStatusAndControlRegister) -> &mut PortStatusAndControlRegister {
            // set 0xxx ...x 0000 000. ..00 00.. ...0 xx0x
        
            portsc
                .set_0_port_enabled_disabled()
                .set_0_connect_status_change()
                .set_0_port_enabled_disabled_change()
                .set_0_warm_port_reset_change()
                .set_0_over_current_change()
                .set_0_port_reset_change()
                .set_0_port_link_state_change()
                .set_0_port_config_error_change()
        }

        assert!(i > 0);
        self.regs.port_register_set_array.index(i - 1)
            .fields().portsc()
            .update(|mut portsc|{
                f(portsc_protect(&mut portsc));
                portsc
            });
    }

    
}

// Basic Command ring and Port configuration functions.
impl<L, A> Controller<'_, L, A>
where
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    // Push the block into the cmd ring and ring doorbell 0.
    fn push_cmd(&self, block: command::TRB) {
        self.cmd_ring.borrow_mut().push(block);
        
        // Ring doorbell 0.
        self.regs.doorbell_array.index(0).update(|mut doorbell| {
            *doorbell.set_doorbell_target(0)
                .set_doorbell_stream_id(0)
        });
    }

    fn reset_port(&mut self, port_id: usize) {
        if !self.port_is_connected(port_id) {
            return;
        }

        if self.addressing_port != 0 {
            self.port_cfg_phase[port_id - 1] = PortConfigPhase::WaitingAddressed;

            // waiting for other addressing port get done
        } else {
            assert!(
                [PortConfigPhase::NotConnected, PortConfigPhase::WaitingAddressed].contains(&self.port_cfg_phase[port_id - 1])
            );

            self.addressing_port = port_id;
            self.port_cfg_phase[port_id - 1] = PortConfigPhase::ResettingPort;

            self.protected_update_portsc_at(port_id, |portsc| {
                portsc
                    // .set_port_power() // rw 1
                    .set_port_reset() // rw1s 1
                    .clear_port_link_state_write_strobe() // rw 0
                    .clear_connect_status_change() // rw1c 1
                ;
            });

            // loop while port reset bit is consumed
            while self.portsc_at(port_id).port_reset() {}
        }
    }

    fn enable_slot(&mut self, port_id: usize) {
        if !self.port_is_enabled(port_id) { return; }
        if !self.port_is_port_reset_changed(port_id) { return; }

        // clear port reset change bit
        self.protected_update_portsc_at(port_id, |portsc| {
            portsc.clear_port_reset_change();
        });

        self.port_cfg_phase[port_id - 1] = PortConfigPhase::EnablingSlot;

        self.push_cmd(command::EnableSlot::new().into());
    }

    fn address_device(&mut self, port_id: usize, slot_id: usize) {
        let port_speed = self.port_speed(port_id);

        let max_packet_size: u16 = match port_speed {
            4 => 512, // Super Speed
            3 => 64, // High Speed
            _ => 8,
        };

        let db = self.regs.doorbell_array.index(slot_id);

        let entry = self.dev_mgr.alloc_entry(slot_id, self.use_64byte_context, db);
        let bus = &entry.bus;

        let tr_buf = bus.alloc_tr(EndpointAddress::control(), 32);

        bus.reset_input_ctx();

        // initialize slot context.
        bus.use_input_slot_ctx(|slot_ctx| {
            slot_ctx.set_route_string(0)
                .set_root_hub_port_number(port_id as u8)
                .set_context_entries(1)
                .set_speed(port_speed)
            ;
        });

        // initialize EP0 context.
        bus.use_input_ep_ctx(EndpointAddress::control(), |ep0_ctx| {
            ep0_ctx.set_endpoint_type(context::EndpointType::Control)
                .set_max_packet_size(max_packet_size)
                .set_max_burst_size(0)
                .set_tr_dequeue_pointer(tr_buf as usize as u64)
                .set_dequeue_cycle_state()
                .set_interval(0)
                .set_max_primary_streams(0)
                .set_mult(0)
                .set_error_count(3)
                .set_endpoint_state(context::EndpointState::Disabled)
            ;
        });

        self.port_cfg_phase[port_id - 1] = PortConfigPhase::AddressingDevice;

        let input_ctx_ptr = bus.input_ctx_ptr();
        self.push_cmd(
            (*command::AddressDevice::new()
                .set_slot_id(slot_id as u8)
                .set_input_context_pointer(input_ctx_ptr as usize as u64)
            ).into()
        );
    }

    fn initialize_device(&mut self, port_id: usize, slot_id: usize) {
        let entry = self.dev_mgr.entry_at(slot_id).unwrap();
        let bus = &entry.bus;

        entry.device.borrow_mut()
            .start_init(bus);

        self.port_cfg_phase[port_id - 1] = PortConfigPhase::InitializingDevice;
    }

    fn complete_configuration(&mut self, port_id: usize, slot_id: usize) {
        let entry = self.dev_mgr.entry_at(slot_id).unwrap();
        let bus = &entry.bus;
        let class_drivers = &entry.class_drivers;

        entry.device.borrow_mut()
            .on_endpoints_configured(bus, class_drivers);

        self.port_cfg_phase[port_id - 1] = PortConfigPhase::Configured;
    }
}

// Event handler functions.
impl<L, A> Controller<'_, L, A>
where
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    fn on_port_status_change(&mut self, port_id: usize) {
        match self.port_cfg_phase[port_id - 1] {
            PortConfigPhase::NotConnected => self.reset_port(port_id),
            PortConfigPhase::ResettingPort => self.enable_slot(port_id),
            _ => panic!("Invalid Phase"),
        };
    }

    fn on_transfer(&mut self, te: event::TransferEvent) {
        let slot_id = te.slot_id() as usize;

        let entry = self.dev_mgr.entry_at(slot_id).unwrap();
        let bus = &entry.bus;
        let class_drivers = &entry.class_drivers;
        let ep_configs = &entry.ep_configs;

        // bus on transfer.
        {
            if ![
                event::CompletionCode::Success,
                event::CompletionCode::ShortPacket
            ].contains(&te.completion_code().unwrap())
            {
                panic!("Transfer Failed");
            }

            let ep_addr = EndpointAddress::from_dci(te.endpoint_id());

            let issuer_pos = te.trb_pointer() as usize as *const transfer::TRB;
            let issuer = unsafe {
                issuer_pos.read_volatile()
            };

            if let Ok(normal_trb) = transfer::Normal::try_from(issuer) {
                // The issuer is normal TRB.
                let buf = unsafe {
                    core::slice::from_raw_parts_mut(
                        normal_trb.data_buffer_pointer() as usize as *mut u8,
                        (normal_trb.trb_transfer_length() - te.trb_transfer_length()) as usize
                    )
                };
                
                entry.device.borrow_mut()
                    .on_normal_completed(bus, class_drivers, ep_addr, buf);
            } else if let Some(setup_stage_trb) = bus.associated_setup_stage(issuer_pos) {
                // The issuer has associated Setup Stage TRB.
                let req = SetupRequest::from_setup_stage_trb(setup_stage_trb);

                let buf = unsafe {
                    if let Ok(data_stage_trb) = transfer::DataStage::try_from(issuer) {
                        core::slice::from_raw_parts_mut(
                            data_stage_trb.data_buffer_pointer() as usize as *mut u8,
                            (data_stage_trb.trb_transfer_length() - te.trb_transfer_length()) as usize
                        )
                    } else { // this branch should be only called for Status Stage TRB.
                        core::slice::from_raw_parts_mut(core::ptr::NonNull::<u8>::dangling().as_ptr(), 0)
                    }
                };

                entry.device.borrow_mut()
                    .on_control_completed(
                        bus,
                        class_drivers,
                        ep_configs,
                        ep_addr, req, buf
                    );

                // entry.device.borrow_mut()
                //     .on_control_completed(
                //         bus,
                //         class_drivers,
                //         ep_configs,
                //         ep_addr, req, buf
                //     );
            } else {
                panic!("No corresponding Setup Stage for the issuer");
            }
        }

        let port_id = bus.port_id();
        
        // configure endpoints
        if entry.device.borrow().is_configured() && self.port_cfg_phase[port_id - 1] == PortConfigPhase::InitializingDevice {
            bus.reset_input_ctx();
            bus.copy_slot_ctx();
            bus.use_input_slot_ctx(|slot_ctx| {
                slot_ctx.set_context_entries(31);
            });

            let port_speed = self.port_speed(port_id);

            let convert_interval = match port_speed {
                1 | 2 => |ep_type: EndpointType, interval: u8| {
                    if ep_type == EndpointType::Isochronous { interval + 2 }
                    else {
                        let msb = (0..=7).rev().find(|&b| {
                            interval & (1u8 << b) != 0
                        }).unwrap_or(0);
                        msb + 3
                    }
                }, // FullSpeed | HighSpeed
                _ => |_, interval: u8| { interval - 1 }
            };

            for ep_config in entry.ep_configs.borrow().iter() {
                let tr_buf = bus.alloc_tr(ep_config.addr, 32);

                bus.use_input_ep_ctx(ep_config.addr, |ep_ctx| {
                    ep_ctx.set_endpoint_type(ep_config.ep_type_with_dir())
                        .set_max_packet_size(ep_config.max_packet_size)
                        .set_interval(convert_interval(ep_config.ep_type(), ep_config.interval))
                        .set_average_trb_length(1)
                        .set_tr_dequeue_pointer(tr_buf as usize as u64)
                        .set_dequeue_cycle_state()
                        .set_max_primary_streams(0)
                        .set_mult(0)
                        .set_error_count(3);
                });
            }

            self.port_cfg_phase[port_id - 1] = PortConfigPhase::ConfiguringEndpoints;

            let input_ctx_ptr = bus.input_ctx_ptr();
            self.push_cmd(
                (*command::ConfigureEndpoint::new()
                    .set_slot_id(slot_id as u8)
                    .set_input_context_pointer(input_ctx_ptr as usize as u64)
                ).into()
            );
        }
    }

    fn on_cmd_complete(&mut self, cc: event::CommandCompletion) {
        let slot_id = cc.slot_id() as usize;
        let issuer = unsafe {
            (cc.command_trb_pointer() as usize as *const command::TRB).read_volatile()
        };

        if let Ok(_) = command::EnableSlot::try_from(issuer) {
            assert!(self.port_cfg_phase[self.addressing_port - 1] == PortConfigPhase::EnablingSlot);

            self.address_device(self.addressing_port, slot_id);
        } else if let Ok(_) = command::AddressDevice::try_from(issuer) {
            let entry = self.dev_mgr.entry_at(slot_id).unwrap();
            let bus = &entry.bus;
            let port_id = bus.port_id();

            assert!(port_id == self.addressing_port);
            assert!(self.port_cfg_phase[port_id - 1] == PortConfigPhase::AddressingDevice);

            // Wake a waiting port, if any.
            self.addressing_port = 0;
            for i in 1..=self.num_ports() {
                if self.port_cfg_phase[i - 1] == PortConfigPhase::WaitingAddressed {
                    self.reset_port(i);
                    break;
                }
            }

            self.initialize_device(port_id, slot_id);
        } else if let Ok(_) = command::ConfigureEndpoint::try_from(issuer) {
            let entry = self.dev_mgr.entry_at(slot_id).unwrap();
            let bus = &entry.bus;
            let port_id = bus.port_id();

            assert!(self.port_cfg_phase[port_id - 1] == PortConfigPhase::ConfiguringEndpoints);

            self.complete_configuration(port_id, slot_id);
        } else {
            unreachable!("Unsupported command TRB.");
        }
    }
}
