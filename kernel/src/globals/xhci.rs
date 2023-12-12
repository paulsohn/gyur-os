
use crate::xhci::{
    self,
    Controller,
    class,
    SupportedClassListeners,
};

use core::cell::OnceCell;
use spin::mutex::Mutex;

pub static XHC: Mutex<OnceCell<Controller<'static, Listeners>>> = Mutex::new(OnceCell::new());

#[allow(unused_must_use)]
pub fn init() {
    let xhci_dev = xhci::get_xhci_dev().unwrap();

    // Configure MSI

    let xhci_mmio_base = xhci_dev.bar0().mm().unwrap();

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
