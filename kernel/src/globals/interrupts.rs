
// use core::cell::LazyCell;
// use spin::mutex::Mutex;
use spin::lazy::Lazy as LazyLock;

use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame
};

const IDT_XHCI: usize = 0x40;
// const IDT_LAPIC_TIMER: usize = 0x41;

static IDT: LazyLock<InterruptDescriptorTable> = LazyLock::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt[IDT_XHCI].set_handler_fn(xhci_handler);
    idt
});

pub fn init(){
    IDT.load();
}

extern "x86-interrupt" fn xhci_handler(stack_frame: InterruptStackFrame) {
    // todo: make `xhc` be static!
    super::XHC.lock().get_mut().unwrap()
        .process_events();
    super::APIC_BASE.end_of_interrupt().signal();
}