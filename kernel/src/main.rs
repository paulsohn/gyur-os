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
    console_print,
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

    {
        let mut console_lock = globals::CONSOLE.get().unwrap().lock();
        let console = core::ops::DerefMut::deref_mut(&mut console_lock);

        use core::fmt::Write;
        console.write_str("Hello 1!\n").unwrap();
        console.write_str("Hello 2!\n").unwrap();
    }

    {
        let mut console_lock = globals::CONSOLE.get().unwrap().lock();
        let console = core::ops::DerefMut::deref_mut(&mut console_lock);

        use core::fmt::Write;
        console.write_str("Hello 3!\n").unwrap();
        console.write_str("Hello 4!\n").unwrap();
    }


    console_println!("Hello, World!");
    console_print!("This is : GYUR OS");
    for i in 0..20 {
        console_println!();
        console_print!("line {:02}", i);
    }
    // for _ in 0..23 {
    //     console_println!();
    // }

    panic!("intentional");

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