
use super::{APIC, XHC};

use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame
};

pub const IDT_VEC_BP: usize = 0x03;
pub const IDT_VEC_XHCI: usize = 0x40;
// const IDT_VEC_LAPIC_TIMER: usize = 0x41;

// This is static to make its lifetime `'static`.
static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init(){
    unsafe {
        IDT[IDT_VEC_BP].set_handler_fn(breakpoint_handler);
        IDT[IDT_VEC_XHCI]
            .set_handler_fn(xhci_handler)
            .set_privilege_level(x86_64::PrivilegeLevel::Ring0)
        ;
        IDT.load();
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    log::info!("Breakpoint occured");
    log::info!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn xhci_handler(stack_frame: InterruptStackFrame) {
    // log::info!("xhci handler invoked");
    {
        XHC.lock().get_mut().unwrap()
            .process_events();
    }

    APIC.end_of_interrupt().signal();
}

