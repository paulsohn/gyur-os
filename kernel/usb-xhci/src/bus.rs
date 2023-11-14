extern crate alloc;

use core::alloc::Allocator;
use core::cell::RefCell;
// use spin::Mutex; // in multi-threaded case, `spin::RwLock` should be used instead of `RefCell`.
use core::marker::PhantomData;

use crate::endpoint::EndpointAddress;
use crate::setup::SetupRequest;

/// A simple USB bus trait.
pub trait USBBus {
    fn control_in(&self, addr: EndpointAddress, req: SetupRequest, buf: &mut [u8]);
    fn control_out(&self, addr: EndpointAddress, req: SetupRequest, buf: &[u8]);

    fn normal_in(&self, addr: EndpointAddress, buf: &mut [u8]);
    fn normal_out(&self, addr: EndpointAddress, buf: &[u8]);

    // fn add_ep_config(&self, ep_config: EndpointConfig);
    // fn ep_configs<'a>(&'a self) -> &'a [EndpointConfig];
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context<T32, T64> {
    C32(T32),
    C64(T64),
}

// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// pub enum XHCIBusState {
//     Invalid,
//     Blank,
//     SlotAssigning,
//     SlotAssigned,
// }
pub struct XHCIBus<A, L>
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
    trs: [RefCell<TransferRing<A>>; 31],

    setup_stage_map: RefCell<ArrayMap<*const Block, *const Block, 16>>,

    ctx:
    Context<
        (RefCell<context::Device32Byte>, RefCell<context::Input32Byte>),
        (RefCell<context::Device64Byte>, RefCell<context::Input64Byte>)
    >,

    _listeners: PhantomData<L>,
}

impl<A, L> XHCIBus<A, L>
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
                RefCell::new( TransferRing::new_uninit(allocator.clone()) )
            }),

            // dev: None,

            setup_stage_map: RefCell::new(ArrayMap::new()),
            ctx: if !use_64byte {
                Context::C32((
                    RefCell::new(context::Device::new_32byte()), RefCell::new(context::Input::new_32byte())
                ))
            } else {
                Context::C64((
                    RefCell::new(context::Device::new_64byte()), RefCell::new(context::Input::new_64byte())
                ))
            },

            _listeners: PhantomData,
        }

        // let dev = Device::new(allocator);
        // bus.dev = Some(RefCell::new(dev));

        // bus
    }

    /// Returns the port id.
    pub fn port_id(&self) -> usize {
        self.dev_ctx_cell().borrow()
            .slot().root_hub_port_number() as usize
    }

    /// Returns the wrapped handler of the device context.
    pub fn dev_ctx_cell(&self) -> &RefCell<dyn context::DeviceHandler> {
        match &self.ctx {
            Context::C32((dev_ctx, _)) => dev_ctx,
            Context::C64((dev_ctx, _)) => dev_ctx,
        }
    }

    /// Returns the wrapped handler of the input context.
    pub fn input_ctx_cell(&self) -> &RefCell<dyn context::InputHandler> {
        match &self.ctx {
            Context::C32((_, input_ctx)) => input_ctx,
            Context::C64((_, input_ctx)) => input_ctx,
        }
    }

    /// Returns the raw pointer of the input context.
    /// 
    /// This methods holds for both 32-byte and 64-byte contexts.
    pub fn input_ctx_ptr(&self) -> *mut context::Input32Byte {
        self.input_ctx_cell().as_ptr() as *mut context::Input32Byte
    }

    /// Copy device-slot context into input-slot context.
    pub fn copy_slot_ctx(&self) {
        match &self.ctx {
            Context::C32((dev_ctx, input_ctx)) => {
                context::InputHandler::device_mut(
                    &mut *input_ctx.borrow_mut()
                )
                    .slot_mut().as_mut().clone_from_slice(
                        context::DeviceHandler::slot(&*dev_ctx.borrow()).as_ref()
                    );
            },
            Context::C64((dev_ctx, input_ctx)) => {
                context::InputHandler::device_mut(
                    &mut *input_ctx.borrow_mut()
                )
                    .slot_mut().as_mut().clone_from_slice(
                        context::DeviceHandler::slot(&*dev_ctx.borrow()).as_ref()
                    );
            },
        };
    }

    /// Invalidate all input contexts, by clearing input control context.
    pub fn reset_ctx(&self) {
        let mut input_ctx = self.input_ctx_cell().borrow_mut();

        input_ctx.control_mut().as_mut().fill(0);
    }

    /// Activate the input slot context, and modify with the callback `f`.
    pub fn use_slot_ctx<F>(&self, f: F)
    where
        F: FnOnce(&mut dyn context::SlotHandler)
    {
        let mut input_ctx = self.input_ctx_cell().borrow_mut();

        input_ctx.control_mut().set_add_context_flag(0);
        f(input_ctx.device_mut().slot_mut());
    }

    /// Activate the ep context, and modify with the callback `f`.
    pub fn use_ep_ctx<F>(&self, addr: EndpointAddress, f: F)
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
        // todo : make this borrow `&self` by implementing interior mutability.

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

impl<A, L> USBBus for XHCIBus<A, L>
where
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    fn control_in(&self, addr: EndpointAddress, req: SetupRequest, buf: &mut [u8]) {
        self.with_tr(addr, |tr| {
            if buf.len() > 0 {
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

                self.setup_stage_map.borrow_mut().set(data_pos, setup_pos);
            } else {
                let setup_trb = *req.into_setup_stage_trb()
                    .set_transfer_type(trb::transfer::TransferType::No);
                let setup_pos = tr.push(block!(setup_trb));

                let status_trb = *trb::transfer::StatusStage::new()
                    .set_direction() // set direction to true
                    .set_interrupt_on_completion();
                let status_pos = tr.push(block!(status_trb));

                self.setup_stage_map.borrow_mut().set(status_pos, setup_pos);
            }
        });
    }

    fn control_out(&self, addr: EndpointAddress, req: SetupRequest, buf: &[u8]) {
        self.with_tr(addr, |tr| {
            if buf.len() > 0 {
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

                self.setup_stage_map.borrow_mut().set(data_pos, setup_pos);
            } else {
                let setup_trb = *req.into_setup_stage_trb()
                    .set_transfer_type(trb::transfer::TransferType::No);
                let setup_pos = tr.push(block!(setup_trb));

                let status_trb = *trb::transfer::StatusStage::new()
                    .set_direction() // set direction to true
                    .set_interrupt_on_completion();
                let status_pos = tr.push(block!(status_trb));

                self.setup_stage_map.borrow_mut().set(status_pos, setup_pos);
            }
        });
    }

    fn normal_in(&self, addr: EndpointAddress, buf: &mut [u8]) {
        self.normal_common(addr, buf);
    }

    fn normal_out(&self, addr: EndpointAddress, buf: &[u8]) {
        self.normal_common(addr, buf);
    }

    // fn add_ep_config(&self, ep_config: EndpointConfig) {
    //     self.ep_configs.borrow_mut().push(ep_config);
    // }

    // fn ep_configs<'a>(&'a self) -> &'a [EndpointConfig] {
    //     self.ep_configs.borrow().as_slice()
    // }
}