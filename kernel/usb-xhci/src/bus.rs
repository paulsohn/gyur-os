extern crate alloc;

use core::alloc::Allocator;
use core::cell::RefCell;
// use spin::Mutex; // in multi-threaded case, `spin::RwLock` should be used instead of `RefCell`.
use core::marker::PhantomData;
use alloc::alloc::Global;
use alloc::boxed::Box;

use crate::endpoint::EndpointAddress;
use crate::setup::SetupRequest;

/// A simple USB bus trait.
pub trait USBBus {
    fn control_in(&self, addr: EndpointAddress, req: SetupRequest, buf: &mut [u8]);
    fn control_out(&self, addr: EndpointAddress, req: SetupRequest, buf: &[u8]);

    fn normal_in(&self, addr: EndpointAddress, buf: &mut [u8]);
    fn normal_out(&self, addr: EndpointAddress, buf: &[u8]);
}

use crate::arraymap::ArrayMap;
use crate::class::SupportedClassListeners;

use xhci::accessor::single;
use xhci::accessor::mapper::Identity;
use xhci::registers::doorbell::Doorbell;
use xhci::ring::buf::TransferRing;
use xhci::ring::buf::block::Block;
use xhci::ring::trb;
use xhci::context;

macro_rules! block {
    ($e:expr) => {
        Block::new($e.into_raw())
    }
}

// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// pub enum XHCIBusState {
//     Invalid,
//     Blank,
//     SlotAssigning,
//     SlotAssigned,
// }
pub struct XHCIBus<L, A = Global>
where
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    // /// The Bus state.
    // state: XHCIBusState,

    // ep_configs: RefCell<Vec<EndpointConfig, A>>,

    /// The doorbell register.
    db: RefCell<single::ReadWrite<Doorbell, Identity>>,
    /// The transfer ring.
    trs: [RefCell<Box<TransferRing<A>, A>>; 31],

    setup_stage_map: RefCell<ArrayMap<*const Block, *const Block, 16>>,

    output_ctx: RefCell<Box<dyn context::OutputHandler, A>>,
    input_ctx: RefCell<Box<dyn context::InputHandler, A>>,

    _listeners: PhantomData<L>,
}

impl<L, A> XHCIBus<L, A>
where
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    pub fn new(db: single::ReadWrite<Doorbell, Identity>, use_64byte: bool, allocator: A) -> Self {
        Self {
            // state: XHCIBusState::Blank,
            // ep_configs: RefCell::new(Vec::new_in(allocator)),

            db: RefCell::new(db),
            trs: core::array::from_fn(|_| {
                RefCell::new(Box::new_in(TransferRing::new_uninit(allocator.clone()), allocator.clone()))
            }),

            // dev: None,

            setup_stage_map: RefCell::new(ArrayMap::new()),
            output_ctx: RefCell::new(
                if !use_64byte {
                    Box::new_in(context::Output::new_32byte(), allocator.clone())
                } else {
                    Box::new_in(context::Output::new_64byte(), allocator.clone())
                }
            ),
            input_ctx: RefCell::new(
                if !use_64byte {
                    Box::new_in(context::Input::new_32byte(), allocator.clone())
                } else {
                    Box::new_in(context::Input::new_64byte(), allocator.clone())
                }
            ),

            _listeners: PhantomData,
        }

        // let dev = Device::new(allocator);
        // bus.dev = Some(RefCell::new(dev));

        // bus
    }

    /// Returns the port id.
    pub fn port_id(&self) -> usize {
        self.output_ctx_cell().borrow()
            .device().slot().root_hub_port_number() as usize
    }

    /// Returns the wrapped handler of the output context.
    pub fn output_ctx_cell(&self) -> &RefCell<Box<dyn context::OutputHandler, A>> {
        &self.output_ctx
    }

    /// Returns the raw pointer of the output context.
    /// 
    /// This methods is valid for both 32-byte and 64-byte contexts.
    pub fn output_ctx_ptr(&self) -> *mut context::Output32Byte {
        (&mut **self.output_ctx_cell().borrow_mut()
            as *mut dyn context::OutputHandler
        ).to_raw_parts().0
            as *mut context::Output32Byte
    }

    /// Returns the wrapped handler of the input context.
    pub fn input_ctx_cell(&self) -> &RefCell<Box<dyn context::InputHandler, A>> {
        &self.input_ctx
    }

    /// Returns the raw pointer of the input context.
    /// 
    /// This methods is valid for both 32-byte and 64-byte contexts.
    pub fn input_ctx_ptr(&self) -> *mut context::Input32Byte {
        (&mut **self.input_ctx_cell().borrow_mut()
            as *mut dyn context::InputHandler
        ).to_raw_parts().0        
            as *mut context::Input32Byte
    }

    /// Copy output slot context into input slot context.
    pub fn copy_slot_ctx(&self) {
        self.input_ctx_cell().borrow_mut()
            .device_mut().slot_mut().as_mut()
            .clone_from_slice(
                self.output_ctx_cell().borrow().device().slot().as_ref()
            );
    }

    /// Invalidate all input contexts, by clearing input control context.
    pub fn reset_input_ctx(&self) {
        self.input_ctx_cell().borrow_mut()
            .control_mut().as_mut().fill(0);
    }

    /// Activate the input slot context, and modify with the callback `f`.
    pub fn use_input_slot_ctx<F>(&self, f: F)
    where
        F: FnOnce(&mut dyn context::SlotHandler)
    {
        let mut input_ctx = self.input_ctx_cell().borrow_mut();

        input_ctx.control_mut().set_add_context_flag(0);
        f(input_ctx.device_mut().slot_mut());
    }

    /// Activate the input ep context, and modify with the callback `f`.
    pub fn use_input_ep_ctx<F>(&self, addr: EndpointAddress, f: F)
    where
        F: FnOnce(&mut dyn context::EndpointHandler)
    {
        let mut input_ctx = self.input_ctx_cell().borrow_mut();
        let dci = addr.dci();

        input_ctx.control_mut().set_add_context_flag(dci);
        f(input_ctx.device_mut().endpoint_mut(dci))
    }

    // pub fn state(&self) -> XHCIBusState {
    //     self.state
    // }

    /// Execute closure `f`, which takes a mutable reference of the transfer ring of address `addr`.
    /// 
    /// Ring the doorbell after everything is finished.
    fn with_tr<F: FnOnce(&mut TransferRing<A>)>(&self, addr: EndpointAddress, f: F) {
        let dci = addr.dci();
        if !(1..=31).contains(&dci) {
            panic!("Invalid DCI");
        }

        let mut tr = self.trs[dci - 1].borrow_mut();
        if !tr.is_init() {
            panic!("Uninitialized Transfer Ring");
        }

        // execute the closure.
        f(&mut tr);

        // ring the doorbell.
        self.db.borrow_mut().update_volatile(|db| {
            db.set_doorbell_target(dci as u8)
                .set_doorbell_stream_id(0);
        });
    }

    /// Allocate a buffer to the transfer ring in `addr`, and get the buffer address.
    pub fn alloc_tr(&self, addr: EndpointAddress, buf_size: usize) -> *const Block {
        let dci = addr.dci();
        if !(1..=31).contains(&dci) {
            panic!("Invalid DCI");
        }

        let mut tr = self.trs[dci - 1].borrow_mut();
        if !tr.is_init() {
            tr.add_segment(buf_size);
        }
        unsafe {
            tr.get_buf_ptr(0)
        }
    }

    fn normal_common(&self, addr: EndpointAddress, buf: &[u8]) {
        self.with_tr(addr, |tr| {
            let normal_trb = *trb::transfer::Normal::new()
                .set_data_buffer_pointer(buf.as_ptr() as usize as _)
                .set_trb_transfer_length(buf.len() as _)
                .set_interrupt_on_short_packet()
                .set_interrupt_on_completion();

            tr.push(block!(normal_trb));
        });
    }

    pub fn associated_setup_stage(&self, issuer_pos: *const Block) -> Option<trb::transfer::SetupStage> {
        self.setup_stage_map.borrow_mut().take(issuer_pos)
            .map(|setup_stage_pos| unsafe {
                let block = setup_stage_pos.read_volatile();
                trb::transfer::SetupStage::try_from(block.into_raw()).unwrap()
            })
    }

}

impl<L, A> USBBus for XHCIBus<L, A>
where
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    fn control_in(&self, addr: EndpointAddress, req: SetupRequest, buf: &mut [u8]) {
        if buf.len() > 0 {
            self.with_tr(addr, |tr| {
                let setup_trb = *req.into_setup_stage_trb()
                    .set_transfer_type(trb::transfer::TransferType::In);
                let setup_pos = tr.push(block!(setup_trb));

                let data_trb = *trb::transfer::DataStage::new()
                    .set_data_buffer_pointer(buf.as_ptr() as usize as _)
                    .set_trb_transfer_length(buf.len() as _)
                    .set_td_size(0)
                    .set_direction(trb::transfer::Direction::In)
                    .set_interrupt_on_completion();
                let data_pos = tr.push(block!(data_trb));

                let status_trb = trb::transfer::StatusStage::new();
                let _status_pos = tr.push(block!(status_trb));

                self.setup_stage_map.borrow_mut().set(&data_pos, setup_pos);
            });
        } else {
            self.with_tr(addr, |tr| {
                let setup_trb = *req.into_setup_stage_trb()
                    .set_transfer_type(trb::transfer::TransferType::No);
                let setup_pos = tr.push(block!(setup_trb));

                let status_trb = *trb::transfer::StatusStage::new()
                    .set_direction() // set direction to true
                    .set_interrupt_on_completion();
                let status_pos = tr.push(block!(status_trb));

                self.setup_stage_map.borrow_mut().set(&status_pos, setup_pos);
            });
        }
    }

    fn control_out(&self, addr: EndpointAddress, req: SetupRequest, buf: &[u8]) {
        if buf.len() > 0 {
            self.with_tr(addr, |tr| {
                let setup_trb = *req.into_setup_stage_trb()
                    .set_transfer_type(trb::transfer::TransferType::Out);
                let setup_pos = tr.push(block!(setup_trb));

                let data_trb = *trb::transfer::DataStage::new()
                    .set_data_buffer_pointer(buf.as_ptr() as usize as _)
                    .set_trb_transfer_length(buf.len() as _)
                    .set_td_size(0)
                    .set_direction(trb::transfer::Direction::Out)
                    .set_interrupt_on_completion();
                let data_pos = tr.push(block!(data_trb));

                let status_trb = *trb::transfer::StatusStage::new()
                    .set_direction();
                let _status_pos = tr.push(block!(status_trb));

                self.setup_stage_map.borrow_mut().set(&data_pos, setup_pos);
            });
        } else {
            self.with_tr(addr, |tr| {
                let setup_trb = *req.into_setup_stage_trb()
                    .set_transfer_type(trb::transfer::TransferType::No);
                let setup_pos = tr.push(block!(setup_trb));
    
                let status_trb = *trb::transfer::StatusStage::new()
                    .set_direction() // set direction to true(in)
                    .set_interrupt_on_completion();
                let status_pos = tr.push(block!(status_trb));
    
                self.setup_stage_map.borrow_mut().set(&status_pos, setup_pos);
            });
        }
    }

    fn normal_in(&self, addr: EndpointAddress, buf: &mut [u8]) {
        self.normal_common(addr, buf);
    }

    fn normal_out(&self, addr: EndpointAddress, buf: &[u8]) {
        self.normal_common(addr, buf);
    }
}