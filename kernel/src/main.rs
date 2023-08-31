#![allow(dead_code)]
#![no_std]
#![no_main]

extern crate shared;

use core::arch::asm;
use core::panic::PanicInfo;
use core::{writeln, fmt::Write};

use shared::{
    FrameBufferInfo,
};
use kernel::{ ColorCode, Screen, Console };

#[no_mangle]
pub extern "sysv64" fn _start (
    frame_buffer_info: FrameBufferInfo
) -> ! {
    // initialize screen before we make a console
    let mut screen = Screen::from(frame_buffer_info);
    screen.write_rect((0,0),(screen.hor_res,screen.ver_res), ColorCode::YELLOW);
    screen.write_rect((0,0),(200,100),ColorCode::GREEN);

    // // ascii printable characters
    // let mut curx = 0usize;
    // for ch in 0x21..=0x7eu8 {
    //     screen.write_ascii( (curx, 48), ch, ColorCode::BLACK, None);
    //     curx += 8;
    // }

    // screen.write_str((0, 64), "Hello, world!", ColorCode::BLUE);

    let mut console = Console::new(screen);
    for i in 0..28 {
        writeln!(console, "line {:02}", i).unwrap();
    }
    // console.write_ascii(b'A');
    // console.write_ascii(b'B');
    console.write_str("Hello, world!\n").unwrap();

    halt();
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    unsafe{
        // handy way to indicate panic, if QEMU debugger is enabled
        asm!("mov r11, 0xDEAD");
    } 
    halt()
}

fn halt() -> ! {
    loop{
        unsafe{ asm!("hlt"); }
    }
}