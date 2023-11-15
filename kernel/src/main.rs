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

    // usb::usb_xhci::controller::Controller,

    globals,
    console_print,
    console_println
};

use kernel::allocator::Bump;
use usb_xhci::controller::Controller;
use usb_xhci::class::{
    KeyboardReport,
    MouseReport,
    SupportedClassListeners
};


fn keyboard_listener(report: KeyboardReport) {
    console_println!("Keyboard Report) modifier : {}", report.modifier);
}

fn mouse_listener(report: MouseReport) {
    console_println!("Mouse Report) {}, {}, {}", report.buttons, report.x, report.y);
}

struct Listeners;
impl SupportedClassListeners for Listeners {
    fn keyboard() -> &'static fn(KeyboardReport) {
        &keyboard_listener
    }
    fn mouse() -> &'static fn(MouseReport) {
        &mouse_listener
    }
}

#[no_mangle]
pub extern "sysv64" fn _start (
    frame_buffer_info: FrameBuffer
) -> ! {
    // initialize globals
    globals::init(
        frame_buffer_info
    );

    console_println!("Hello, GYUR OS!");

    {
        let mut screen_cell = globals::SCREEN.lock();
        let screen = screen_cell.get_mut().unwrap();

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

    use kernel::pci::{ Devices, Device, Bar };
    let devices = Devices::scan().unwrap();
    for dev in devices.as_slice() {
        console_println!("{}.{}.{}.: vend {:04x}, class {:06x}, head {:02x}", dev.bus(), dev.slot_fun().0, dev.slot_fun().1, dev.vendor_id(), dev.class_code().code(), dev.header_type());
    }

    let xhc_dev = devices.as_slice().iter().find(|&dev| {
        dev.class_code().match_base_sub_interface(0x0c, 0x03, 0x30)
    });
    if let Some(xdev) = xhc_dev {
        console_println!("xHC has been found: {}.{}.{}.", xdev.bus(), xdev.slot_fun().0, xdev.slot_fun().1);

        let switch_ehci_to_xhci = (xdev.vendor_id() == 0x8086)
            && devices.as_slice().iter().find(|&dev| {
                dev.class_code().match_base_sub_interface(0x0c, 0x03, 0x20)
            }).is_some();
        if switch_ehci_to_xhci {
            // read and write `xdev`
            xdev.read_write_offset(0xdc, 0xd8); // Superspeed Ports
            xdev.read_write_offset(0xd4, 0xd0); // eHCi to xHCi ports
            console_println!("Switched eHCi to xHCi");
        }

        if let Bar::MM(mmio_base) = xdev.bar0() {
            console_println!("BAR xHC MMIO base: {:08x}", mmio_base);

            // let mut xhc = Controller::new(mmio_base);
            // xhc.run();
            
            let mut xhc: Controller<_, Listeners> = Controller::new(mmio_base, Bump::new());
            xhc.run();

            loop {
                xhc.process_events();
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