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
    // unsafe {
    //     asm!(
    //         "movq xmm0, {0}",
    //         "movq xmm2, {1}",
    //         "movq xmm4, {2}",
    //         "movq xmm6, {3}",
    //         in(reg) frame_buffer_info.base,
    //         in(reg) frame_buffer_info.stride,
    //         in(reg) frame_buffer_info.hor_res,
    //         in(reg) frame_buffer_info.ver_res,
    //     )
    // }

    let mut screen = Screen::from(frame_buffer_info);
    for x in 0..screen.hor_res {
        for y in 0..screen.ver_res {
            // screen.write_pixel( (x,y), ColorCode::YELLOW );
            screen.write_pixel(
                (x,y),
                ColorCode::rgb(
                    u8::try_from(x % 256).unwrap(),
                    u8::try_from(x % 256).unwrap(),
                    u8::try_from(y % 256).unwrap()
                )
            );
        }
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