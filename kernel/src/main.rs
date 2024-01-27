// #![allow(dead_code)]
#![no_std]
#![no_main]

extern crate shared;

use core::panic::PanicInfo;

use kernel::{
    globals,
    // console_print,
    console_println
};

const KERNEL_MAIN_STACK_SIZE: usize = 0x100000; // 1MB
static KERNEL_MAIN_STACK: [u8; KERNEL_MAIN_STACK_SIZE] = [0; KERNEL_MAIN_STACK_SIZE];

#[inline]
pub fn relocate_stack(){
    // Relocate kernel stack.
    // This should preceed over any function calls, and the function itself SHOULD BE inline.
    unsafe {
        let kernel_main_stack_top = (&KERNEL_MAIN_STACK as *const u8)
            .add(KERNEL_MAIN_STACK_SIZE);
        core::arch::asm!(
            "mov rsp, {}",
            in(reg) kernel_main_stack_top
        );
    }
}

#[no_mangle]
pub extern "sysv64" fn _start (
    mmap: shared::uefi_memory::MemoryMap<'static>,
    args: shared::KernelArgs,
) -> ! {
    relocate_stack();

    // initialize globals
    globals::init(args);

    for (i, desc) in mmap.entries().enumerate() {
        log::info!(
            "{},{:X},{:?},{:08X},{:X},{:X}",
            i, desc.ty.0, desc.ty, desc.phys_start, desc.page_count, desc.att.bits()
        );
    }

    // log::info!("Hello, GYUR OS!");

    loop {
        // Dequeuing should be occured in critical section
        // and no interrupts should happen during it.
        // We are using lock-free MPMC queue, but in general situations, `x86_64::instructions::interrupts::disable();` and `x86_64::instructions::interrupts::enable()` should wrap this dequeueing.

        match globals::MSG_QUEUE.dequeue() {
            Some(kernel::message::Message::XHCIInterrupt) => {
                globals::XHC.lock().get_mut().unwrap()
                    .process_events();
            },
            None => halt(),
        }
    }
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    // handy way to indicate panic, if QEMU debugger is enabled
    unsafe {
        core::arch::asm!("mov r11, 0xDEAD");
    }

    console_println!("{}", info);
    loop { halt() }
}

fn halt() { // should be `!` return type, but this doesn't seem to implement that..
    x86_64::instructions::hlt();
}