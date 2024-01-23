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

#[no_mangle]
pub extern "sysv64" fn _start (
    frame_buffer_info: shared::frame_buffer::FrameBuffer,
    memory_map: shared::memory_map::MemoryMap<'static>,
) -> ! {
    // initialize globals
    globals::init(
        frame_buffer_info
    );

    for (i, desc) in memory_map.entries().enumerate() {
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
    unsafe{ core::arch::asm!("mov r11, 0xDEAD"); }

    console_println!("{}", info);
    loop { halt() }
}

fn halt() { // should be `!` return type, but this doesn't seem to implement that..
    x86_64::instructions::hlt();
}