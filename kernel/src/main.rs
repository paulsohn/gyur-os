#![allow(dead_code)]
#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

#[no_mangle]
fn main() {
    
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> !{
    halt()
}

fn halt() -> !{
    loop{
        unsafe{ asm!("hlt"); }
    }
}