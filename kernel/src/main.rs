// #![allow(dead_code)]
#![no_std]
#![no_main]

extern crate shared;

use core::panic::PanicInfo;

use shared::FrameBufferInfo;
use kernel::{
    // screen::ColorCode,
    // console::Console,

    globals,
    console_print,
    console_println
};

#[no_mangle]
pub extern "sysv64" fn _start (
    frame_buffer_info: FrameBufferInfo
) -> ! {
    // initialize globals
    globals::init(
        frame_buffer_info
    );

    console_println!("Hello, World!");
    console_print!("This is : GYUR OS");
    for i in 0..25 {
        console_println!();
        console_print!("line {:02}", i);
    }

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
    loop{
        x86_64::instructions::hlt();
    }
}