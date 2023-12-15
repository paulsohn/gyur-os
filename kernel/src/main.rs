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

    // {
    //     let mut screen_cell = globals::SCREEN.lock();
    //     let screen = screen_cell.get_mut().unwrap();

    //     screen.render_cursor();
    // }

    // let mut xhc = globals::XHC.lock().get().unwrap();
    // loop {
    //     xhc.process_events();
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