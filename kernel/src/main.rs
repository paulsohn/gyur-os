#![allow(dead_code)]
#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    halt();
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> !{
    halt()
}

fn halt() -> !{
    loop{
        unsafe{ asm!("hlt"); }
    }
}