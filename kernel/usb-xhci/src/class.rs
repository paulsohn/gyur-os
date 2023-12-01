extern crate alloc;

use core::alloc::Allocator;
use core::marker::PhantomData;
// use alloc::alloc::Global;
use alloc::boxed::Box;

use bitvec::prelude::*;

use crate::endpoint::{EndpointAddress, EndpointConfig, EndpointDirectedType};
use crate::setup::{/* request_code, */ requests, SetupRequest};
// use crate::descriptor::InterfaceDescriptorBody;

use crate::bus::USBBus;

/// USB class driver.
pub trait USBClass<B: USBBus> {
    fn set_endpoint(&mut self, configs: &[EndpointConfig]);
    fn on_endpoints_configured(&mut self, bus: &B);
    fn on_control_completed(&mut self, bus: &B, addr: EndpointAddress, req: SetupRequest, buf: &mut [u8]); // should only process if the recent setup request matches with `req`
    fn on_normal_completed(&mut self, bus: &B, addr: EndpointAddress, buf: &mut [u8]);
}

/// temporary void class
// impl<B: USBBus> USBClass<B> for () {
//     fn set_endpoint(&mut self, _configs: &[EndpointConfig]) {}
//     fn on_endpoints_configured(&mut self, _bus: &B) {}
//     fn on_control_completed(&mut self, _bus: &B, _addr: EndpointAddress, _req: SetupRequest, _buf: &mut [u8]) {}
//     fn on_normal_completed(&mut self, _bus: &B, _addr: EndpointAddress, _buf: &mut [u8]) {}
// }

// pub struct USBCDCClass<B>
// where
//     B: USBBus,
// {
//     ep_interrupt_in: EndpointAddress,
//     ep_bulk_in: EndpointAddress,
//     ep_bulk_out: EndpointAddress,

//     _bus: PhantomData<B>,
// }

// impl<B> USBCDCClass<B>
// where
//     B: USBBus,
// {
//     pub fn new() -> Self {
//         Self {
//             ep_interrupt_in: EndpointAddress::from_byte(0),
//             ep_bulk_in: EndpointAddress::from_byte(0),
//             ep_bulk_out: EndpointAddress::from_byte(0),
//             _bus: PhantomData
//         }
//     }
// }

// impl<B> USBClass<B> for USBCDCClass<B>
// where
//     B: USBBus,
// {
//     fn set_endpoint(&mut self, configs: &[EndpointConfig]) {
//         for cfg in configs.iter() {
//             match cfg.ep_type_with_dir() {
//                 EndpointDirectedType::InterruptIn => {
//                     self.ep_interrupt_in = cfg.addr;
//                 },
//                 EndpointDirectedType::BulkIn => {
//                     self.ep_bulk_in = cfg.addr;
//                 },
//                 EndpointDirectedType::BulkOut => {
//                     self.ep_bulk_out = cfg.addr;
//                 },
//                 _ => {},
//             }
//         }
//     }

//     fn on_endpoints_configured(&mut self, _bus: &B) {
//         // pass
//     }

//     fn on_control_completed(&mut self, _bus: &B, _addr: EndpointAddress, _req: SetupRequest, _buf: &mut [u8]) {
//         // pass
//     }

//     fn on_normal_completed(&mut self, _bus: &B, _addr: EndpointAddress, _buf: &mut [u8]) {
//         todo!("Send Serial, Receive Serial, Set Line Coding..");
//     }
// }

pub struct USBHIDClass<B, P>
where
    B: USBBus,
    P: Packet,
{
    ep_interrupt_in: EndpointAddress,
    ep_interrupt_out: EndpointAddress,

    if_index: u16,

    last_req: SetupRequest,

    packet: core::mem::MaybeUninit<P>,
    prev: P::Info,

    listener: fn(P::Report),
    _marker: PhantomData<B>,
}

impl<B, P> USBHIDClass<B, P>
where
    B: USBBus,
    P: Packet,
{
    pub fn new(
        if_index: u16,
        listener: fn(P::Report),
    ) -> Self {
        Self {
            ep_interrupt_in: EndpointAddress::from_byte(0),
            ep_interrupt_out: EndpointAddress::from_byte(0),

            if_index,

            last_req: Default::default(),
            
            packet: core::mem::MaybeUninit::uninit(),
            prev: Default::default(),

            listener,
            _marker: PhantomData,
        }
    }

    fn packet_buf(&self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(&self.packet as *const _ as *mut _, core::mem::size_of::<P>())
        }
    }
}

impl<B, P> USBClass<B> for USBHIDClass<B, P>
where
    B: USBBus,
    P: Packet,
{
    fn set_endpoint(&mut self, configs: &[EndpointConfig]) {
        for cfg in configs.iter() {
            match cfg.ep_type_with_dir() {
                EndpointDirectedType::InterruptIn => {
                    self.ep_interrupt_in = cfg.addr;
                },
                EndpointDirectedType::InterruptOut => {
                    self.ep_interrupt_out = cfg.addr;
                },
                _ => {},
            }
        }
    }

    fn on_endpoints_configured(&mut self, bus: &B) {
        let req = requests::set_protocol(self.if_index);

        bus.control_out(
            EndpointAddress::control(),
            req,
            unsafe {
                core::slice::from_raw_parts_mut(core::ptr::NonNull::dangling().as_ptr(), 0)
            },
        );

        self.last_req = req;
    }

    fn on_control_completed(&mut self, bus: &B, _addr: EndpointAddress, req: SetupRequest, _buf: &mut [u8]) {
        if self.last_req != req { return; }
        self.last_req = Default::default();

        bus.normal_in(
            self.ep_interrupt_in,
            self.packet_buf()
        );
    }

    fn on_normal_completed(&mut self, bus: &B, addr: EndpointAddress, _buf: &mut [u8]) {
        if addr != self.ep_interrupt_in { return; }
        // if !addr.is_in() { return; }

        // notify report to the listener.
        let (report, info) = (unsafe { self.packet.assume_init_read() }).create_report(self.prev);

        (self.listener)(report);
        self.prev = info;

        bus.normal_in(
            self.ep_interrupt_in,
            self.packet_buf()
        );
    }
}


/// Trait for packet types.
/// HID device types are distinguished with the packet types they use.
pub trait Packet: /* Sized + */ Clone + Copy + 'static {
    /// The associated report type.
    type Report: Clone + Copy + 'static;

    /// The context info type.
    /// A report can be constructed from context info and received packet. After constructing a report, the context should be mutated.
    type Info: Clone + Copy + Default;

    /// The method to construct report and change context info.
    fn create_report(&self, prev: Self::Info) -> (Self::Report, Self::Info);

    // fn create_report_from_buf(buf: &[u8], prev: &mut Self::Info) -> Self::Report {
    //     let packet = unsafe {
    //         core::mem::transmute::<_, *const Self>(buf.as_ptr()).read()
    //     };
    //     let (report, info) = packet.create_report(*prev);

    //     *prev = info;
    //     report
    // }
}

/// A (boot) mouse packet.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[repr(C, packed)]
pub struct MousePacket {
    pub buttons: u8,
    pub x: i8,
    pub y: i8,
}
pub type MouseReport = MousePacket;

impl Packet for MousePacket {
    type Report = MouseReport;
    type Info = ();

    fn create_report(&self, prev: Self::Info) -> (Self::Report, Self::Info) {
        (*self, prev)
    }
}

/// A (boot) keyboard packet.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[repr(C, packed)]
pub struct KeyboardPacket {
    pub modifier: u8,
    _reserved: u8,
    pub keys: [u8; 6]
}

/// Keyboard bitset.
pub type KeyboardBitSet = BitArr!(for 256, in u32, Msb0);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct KeyboardReport {
    pub modifier: u8,
    pub cur_keys: KeyboardBitSet,
    pub prev_keys: KeyboardBitSet,
}

impl Packet for KeyboardPacket {
    type Report = KeyboardReport;
    type Info = KeyboardBitSet;

    fn create_report(&self, prev_keys: Self::Info) -> (Self::Report, Self::Info) {
        let modifier = self.modifier;
        let mut cur_keys: KeyboardBitSet = Default::default();
        for &key in self.keys.iter() {
            // cur_keys[key as usize] = true;
            cur_keys.set(key as usize, true);
        }
        cur_keys.set(0, false); // 0 indicates no event

        (
            Self::Report {
                modifier,
                cur_keys,
                prev_keys,
            },
            cur_keys
        )
    }
}

/// A marker trait for listener configuration.
pub trait SupportedClassListeners: 'static {
    fn keyboard() -> fn(KeyboardReport);
    fn mouse() -> fn(MouseReport);
}

pub fn new_class_from_interface<'b, B, L, A /* = Global */>(
    base: u8, sub: u8, protocol: u8,
    index: u8,
    allocator: A
) -> Option<Box<dyn USBClass<B> + 'b, A>>
where
    B: USBBus + 'b,
    A: Allocator,
    L: SupportedClassListeners,
{
    match (base, sub, protocol) {
        // (2, _, _) => { // cdc
        //     // todo!() // unsupported currently.
        //     None
        // },
        (3, 1, 1) => { // keyboard
            Some(Box::new_in(
                USBHIDClass::<B, KeyboardPacket>::new(
                    index as u16,
                    L::keyboard()
                ),
                allocator
            ))
        },
        (3, 1, 2) => { // mouse
            Some(Box::new_in(
                USBHIDClass::<B, MousePacket>::new(
                    index as u16,
                    L::mouse()
                ),
                allocator
            ))
        },
        _ => None
    }
}