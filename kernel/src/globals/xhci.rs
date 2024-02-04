
use crate::xhci::{
    self,
    Controller,
    class,
    SupportedClassListeners,
};

use core::cell::OnceCell;
use spin::mutex::Mutex;

use super::interrupts::IDT_VEC_XHCI;

pub static XHC: Mutex<OnceCell<Controller<'static, Listeners>>> = Mutex::new(OnceCell::new());

#[allow(unused_must_use)]
#[inline]
pub fn init() {
    let apic = &*super::APIC;
    log::info!("base {:p} / bsp id {}", apic.base_addr.as_ptr(), apic.id().read().id());

    let xhci_ep_acc = xhci::get_xhci_ep_acc().unwrap();

    // Enable MSI.
    let msi_cap_header_acc = xhci::find_msi_cap_acc(&xhci_ep_acc).unwrap();
    xhci::cfg_msi_fixed_dst(
        &msi_cap_header_acc,
        apic.base_addr,
        apic.id().read().id(), // bootstrap processor LAPIC ID
        IDT_VEC_XHCI as u8,
    );

    // Setup xhc controller.
    let xhci_mmio_base = xhci::read_mmio_base(&xhci_ep_acc).unwrap();
    XHC.lock().get_or_init(|| {
        xhci::setup_xhc_controller(xhci_mmio_base).unwrap()
    });
}

pub struct Listeners;
impl SupportedClassListeners for Listeners {
    fn keyboard() -> fn(class::KeyboardReport) {
        fn keyboard_listener(report: class::KeyboardReport) {
            log::debug!("Keyboard Report) modifier : {}", report.modifier);
        }

        keyboard_listener
    }
    fn mouse() -> fn(class::MouseReport) {
        fn mouse_listener(report: class::MouseReport) {
            log::debug!("Mouse Report) {}, {:?}", report.buttons, report.disp);

            crate::globals::SCREEN.lock().get_mut().unwrap().move_cursor(report.disp.into());
        }

        mouse_listener
    }
}
