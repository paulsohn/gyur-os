
use super::{APIC, MSG_QUEUE};

use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
    PageFaultErrorCode
};
use x86_64::registers::control::Cr2;

// pub const IDT_VEC_BP: usize = 0x03;
// pub const IDT_VEC_PF: usize = 0x0E;
pub const IDT_VEC_XHCI: usize = 0x40;
// const IDT_VEC_LAPIC_TIMER: usize = 0x41;

// This is static to make its lifetime `'static`.
static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init(){
    unsafe {
        IDT.breakpoint.set_handler_fn(breakpoint_handler);
        IDT.page_fault.set_handler_fn(page_fault_handler);
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

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode
) {
    log::error!("Exception: Page Fault");
    log::error!("Accessed Address: {:?}", Cr2::read());

    log::error!("Error Code: {:?}", error_code);
    log::error!("{:#?}", stack_frame);

    panic!("Page Fault Handling not implemented");
}

extern "x86-interrupt" fn xhci_handler(_stack_frame: InterruptStackFrame) {
    MSG_QUEUE.enqueue(
        crate::message::Message::XHCIInterrupt
    )
        .expect("Message Queue Full");

    APIC.end_of_interrupt().signal(); // Do we really need this?
}


