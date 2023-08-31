#![allow(dead_code)]
#![no_std]
#![no_main]

extern crate shared;

use core::arch::asm;
use core::panic::PanicInfo;

use shared::{
    FrameBufferInfo,
};
use kernel::{ ColorCode, Screen };

#[no_mangle]
pub extern "sysv64" fn _start (
    frame_buffer_info: FrameBufferInfo
) -> ! {
    let mut screen = Screen::from(frame_buffer_info);
    for x in 0..screen.hor_res {
        for y in 0..screen.ver_res {
            screen.write_pixel( (x,y), ColorCode::YELLOW );
        }
    }

    for x in 0..200usize {
        for y in 0..100usize {
            screen.write_pixel( (x,y), ColorCode::GREEN );
        }
    }

    let mut curx = 0usize;
    for ch in 0x21..=0x7eu8 {
        screen.write_ascii( (curx, 64), ch, ColorCode::BLACK );
        curx += 8;
    }

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