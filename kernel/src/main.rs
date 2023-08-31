// #![allow(dead_code)]
#![no_std]
#![no_main]

extern crate shared;

use core::arch::asm;
use core::panic::PanicInfo;

use shared::FrameBufferInfo;
use kernel::{
    // screen::ColorCode,
    // console::Console,

    globals,
    // console_print,
    console_println
};

#[no_mangle]
pub extern "sysv64" fn _start (
    frame_buffer_info: FrameBufferInfo
) -> ! {
    // initialize globals
    globals::init_globals(
        frame_buffer_info
    );

    for i in 0..28 {
        console_println!("line {:02}", i);
    }
    console_println!("Hello, world!");

    // panic!("intentional");

    halt();
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe{
        // handy way to indicate panic, if QEMU debugger is enabled
        asm!("mov r11, 0xDEAD");
    }
    console_println!("{}", info);
    halt()
}

fn halt() -> ! {
    loop{
        unsafe{ asm!("hlt"); }
    }
}