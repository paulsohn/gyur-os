use crate::pci::{Devices, Device};
use crate::allocator::global_allocator;

pub use usb_xhci::controller::Controller;
pub use usb_xhci::class::{
    self,
    SupportedClassListeners
};

pub fn get_xhci_dev() -> Option<Device> {
    let devices = Devices::scan().unwrap();
    // for dev in devices.as_slice() {
    //     log::debug!("{}.{}.{}.: vend {:04x}, class {:06x}, head {:02x}", dev.bus(), dev.slot_fun().0, dev.slot_fun().1, dev.vendor_id(), dev.class_code().code(), dev.header_type());
    // }

    devices.as_slice().iter().find(|&dev| {
        dev.class_code().match_base_sub_interface(0x0c, 0x03, 0x30)
    }).map(|dev| *dev)
}

pub fn setup_xhc_controller<L: SupportedClassListeners>(
    xhci_mmio_base: u64
) -> Option<Controller<'static, L>>
{
    let mut xhc = Controller::new(xhci_mmio_base, global_allocator());
    xhc.run();
    xhc.reconfigure_port();

    // after here, use `xhc.process_events()` to process events.

    Some(xhc)
}