// #![allow(dead_code)]
#![no_std]
#![no_main]

extern crate shared;

use core::panic::PanicInfo;

use shared::frame_buffer::FrameBuffer;
use kernel::{
    screen::ColorCode,
    // console::Console,

    cursor::{
        SYSCURSOR_WIDTH,
        SYSCURSOR_HEIGHT,
        SYSCURSOR_SHAPE
    },

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
    for i in 0..20 {
        console_println!();
        console_print!("line {:02}", i);
    }

    {
        let mut screen_lock = globals::SCREEN.lock();
        let screen = screen_lock.get_mut().unwrap();

        let x = 200usize;
        let y = 100usize;

        for xx in x..(x+SYSCURSOR_WIDTH).min(screen.resolution().0) {
            for yy in y..(y+SYSCURSOR_HEIGHT).min(screen.resolution().1) {
                let ch = match SYSCURSOR_SHAPE[yy-y][xx-x] {
                    b'@' => ColorCode::BLACK,
                    b'.' => ColorCode::WHITE,
                    _ => continue, // transparent
                };
                screen.render_pixel((xx, yy), ch);
            }
        }
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
    loop{ x86_64::instructions::hlt(); }
}