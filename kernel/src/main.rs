// #![allow(dead_code)]
#![no_std]
#![no_main]

extern crate shared;

use core::panic::PanicInfo;

use shared::{
    frame_buffer::FrameBuffer
};
use kernel::{
    // screen::ColorCode,
    // console::Console,

    globals,
    console_print,
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

    console_print!("Hello, GYUR OS!");
    for _ in 0..20 {
        console_println!();
        console_print!("line {:02}", i);
    }

    // {
    //     let mut screen_lock = globals::SCREEN.lock();
    //     let screen = screen_lock.get_mut().unwrap();
    // }

    halt();
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    // handy way to indicate panic, if QEMU debugger is enabled
    unsafe{ core::arch::asm!("mov r11, 0xDEAD"); }

    console_println!("{}", info);
    halt()
}

fn halt() -> ! {
    loop{ x86_64::instructions::hlt(); }
}