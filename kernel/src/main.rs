// #![allow(dead_code)]
#![no_std]
#![no_main]

extern crate shared;

use core::panic::PanicInfo;

use shared::frame_buffer::FrameBuffer;
use kernel::{
    globals,
    // console_print,
    console_println
};

#[no_mangle]
pub extern "sysv64" fn _start (
    frame_buffer_info: FrameBuffer
) -> ! {
    // initialize globals
    globals::init(
        frame_buffer_info
    );

    // log::info!("Hello, GYUR OS!");

    loop {
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