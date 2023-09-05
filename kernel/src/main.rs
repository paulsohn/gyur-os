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

    console_println!("Hello, GYUR OS!");

    use kernel::pci::{Device, Devices, vendor_id, class_code};
    let devices = Devices::scan().unwrap();
    for device in devices.as_slice() {
        let vend = vendor_id(device.bus, device.dev, device.fun);
        let class = class_code(device.bus, device.dev, device.fun);
        console_println!("{}.{}.{}.: vend {:04x}, class {:08x}, head {:02x}", device.bus, device.dev, device.fun, vend, class, device.header_type);
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