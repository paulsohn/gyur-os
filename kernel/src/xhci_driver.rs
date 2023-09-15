
use core::num::NonZeroUsize;

// I don't like these 'set-clear' method pairs though.
use xhci::registers::{
    Registers,
    capability::{
        Capability,
        CapabilityParameters1,
        // CapabilityParameters2,
        StructuralParameters1,
        StructuralParameters2,
    },
    operational::{
        Operational,
        UsbCommandRegister,
        UsbStatusRegister,
        ConfigureRegister,
        DeviceContextBaseAddressArrayPointerRegister,
    },
    runtime::{
        InterrupterRegisterSet,
        Interrupter,
        EventRingDequeuePointerRegister,
        EventRingSegmentTableSizeRegister,
        EventRingSegmentTableBaseAddressRegister, InterrupterManagementRegister,
    }
};

use xhci::context::{
    Device as DeviceContext,
    Input as InputContext,
    Slot as SlotContext,

};

use xhci::extended_capabilities::{
    ExtendedCapability,
    List,
    usb_legacy_support_capability::UsbLegacySupport
};

use xhci::ring::trb::{
    command,
    event,
};

use xhci::accessor::Mapper;

struct XhciMapper; // @TODO

impl Mapper for XhciMapper {
    unsafe fn map(&mut self, phys_start: usize, bytes: usize) -> NonZeroUsize {
        todo!()
    }

    fn unmap(&mut self, virt_start: usize, bytes: usize) {
        todo!()
    }
}

impl Clone for XhciMapper {
    fn clone(&self) -> Self {
        todo!()
    }
}

#[repr(C)]
pub struct XhciDevice {
    dev_ctx: DeviceContext<8>,
    input_ctx: InputContext<8>
}

pub struct XhciDeviceManager {
    devices: [XhciDevice; 8],
}



pub struct Controller{
    regs: Registers<XhciMapper>,
    dev_mgr: XhciDeviceManager<XhciMapper>,
    // cmd_ring: Ring<command::Allowed>,
    // ev_ring: EventRing,

    // primary interrupter is interrupter index 0
}

impl Controller {
    pub fn new(mmio_base: u64) -> Self {
        const MAX_DEVICE_SLOTS: u8 = 8;

        let mapper: XhciMapper = todo!("implement a mapper");

        let regs = unsafe {
            Registers::new(mmio_base as usize, mapper.clone())
        };

        let cap = &mut regs.capability;
        let op = &mut regs.operational;

        // primary interrupter.
        let mut interrupter = regs.interrupter_register_set.interrupter_mut(0);

        //******** Initialization process begin ********//

        // init device manager.
        let dev_mgr: XhciDeviceManager<XhciMapper> = todo!("MAX_DEVICE_SLOTS");

        // Request Host Controller ownership
        'req_own: {
            let hccparams1 = cap.hccparams1.read_volatile();

            let ext_list: List<XhciMapper> = unsafe {
                List::new(
                    mmio_base as usize,
                    hccparams1,
                    mapper
                ).unwrap()
            };

            if let Some(ext_cap_usb) = ext_list.into_iter().find_map(|&ext| {
                if let Ok(ExtendedCapability::<XhciMapper>::UsbLegacySupport(x)) = ext {
                    Some(x)
                } else { None }
            }) {
                let mut legsup = ext_cap_usb.usblegsup.read_volatile();
                if legsup.hc_os_owned_semaphore() { break 'req_own; }

                legsup.set_hs_os_owned_semaphore();
                ext_cap_usb.usblegsup.write_volatile(legsup);

                loop { // wait until os gets controller ownership.
                    let legsup = ext_cap_usb.usblegsup.read_volatile();
                    if legsup.hc_os_owned_semaphore()
                    && !legsup.hc_bios_owned_semaphore() {
                        break;
                    }
                }
            }
        }

        // disable interrupt for controller and stop
        {
            let mut usbcmd = op.usbcmd.read_volatile();
            usbcmd.clear_interrupter_enable();
            usbcmd.clear_host_system_error_enable();
            usbcmd.clear_enable_wrap_event();

            let usbsts = op.usbsts.read_volatile();
            if usbsts.hc_halted() {
                usbcmd.clear_run_stop(); // stop
            }

            op.usbcmd.write_volatile(usbcmd);
            loop { // wait until hc has halted
                let usbsts = op.usbsts.read_volatile();
                if usbsts.hc_halted() { break; }
            }
        }

        // reset controller
        {
            let mut usbcmd = op.usbcmd.read_volatile();
            usbcmd.set_host_controller_reset();

            op.usbcmd.write_volatile(usbcmd);
            loop { // wait until `hc_reset` bit has been consumed.
                let usbcmd = op.usbcmd.read_volatile();
                if !usbcmd.host_controller_reset() { break; } // is set to false
            }
            loop { // wait until controller is ready.
                let usbsts = op.usbsts.read_volatile();
                if !usbsts.controller_not_ready() { break; }
            }
        }

        // set max device slots
        {
            let mut config = op.config.read_volatile();
            config.set_max_device_slots_enabled(MAX_DEVICE_SLOTS);
            op.config.write_volatile(config);
        }

        // allocate scratchpad buffer arrays.
        {
            let hcsparams2 = cap.hcsparams2.read_volatile();
            let max_sp_buffers = hcsparams2.max_scratchpad_buffers();
            if max_sp_buffers > 0 {
                todo!("allocate scratchpad buffer arrays");
                // let arr = allocArray(max_sp_buffers, 64, 4096);
                // for i in 0..max_sp_buffers { arr[i] = allocMem(4096, 4096, 4096) }
            }
            todo!("pass `arr` into devmgr.");
        }
        
        // set DCBAAP
        {
            let mut dcbaap = op.dcbaap.read_volatile();
            let dcbaa : u64 = todo!("base address from devmgr");
            dcbaap.set(dcbaa);
            op.dcbaap.write_volatile(dcbaap);
        }

        // initialize Command Ring and Event Ring

        let cmd_ring = todo!();
        let ev_ring = todo!();

        // enable interrupt for primary interrupter
        {
            let mut iman = interrupter.iman.read_volatile();
            iman.set_0_interrupt_pending(); // @Todo : want to set to 1
            iman.clear_interrupt_enable();
            interrupter.iman.write_volatile(iman);
        }

        // enable interrupt for controller
        {
            let mut usbcmd = op.usbcmd.read_volatile();
            usbcmd.set_interrupter_enable();
            op.usbcmd.write_volatile(usbcmd);
        }

        Self{
            regs,
            dev_mgr,
            // cmd_ring,
            // ev_ring,
        }
    }

    /// run the controller.
    pub fn run(&mut self) {
        let op = &mut self.regs.operational;

        // set run-stop bit
        {
            let mut usbcmd = op.usbcmd.read_volatile();
            usbcmd.set_run_stop();
            op.usbcmd.write_volatile(usbcmd);

            loop { // wait until hc is running
                let usbsts = op.usbsts.read_volatile();
                if !usbsts.hc_halted() { break; }
            }
        }
    }
}