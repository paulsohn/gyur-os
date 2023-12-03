// #![allow(dead_code)]
#![no_std]
#![no_main]

extern crate shared;

use core::panic::PanicInfo;

use shared::frame_buffer::FrameBuffer;
use kernel::{
    // console::Console,

    allocator::global_allocator,

    globals,
    // console_print,
    console_println
};

use usb_xhci::controller::Controller;
use usb_xhci::class::{
    KeyboardReport,
    MouseReport,
    SupportedClassListeners
};

struct Listeners;
impl SupportedClassListeners for Listeners {
    fn keyboard() -> fn(KeyboardReport) {
        fn keyboard_listener(report: KeyboardReport) {
            log::debug!("Keyboard Report) modifier : {}", report.modifier);
        }

        keyboard_listener
    }
    fn mouse() -> fn(MouseReport) {
        fn mouse_listener(report: MouseReport) {
            log::debug!("Mouse Report) {}, {:?}", report.buttons, report.disp);

            globals::SCREEN.lock().get_mut().unwrap().move_cursor(report.disp.into());
        }

        mouse_listener
    }
}

fn setup_xhc_controller() -> Option<Controller<Listeners>> {
    use kernel::pci::Devices;
    let devices = Devices::scan().unwrap();
    for dev in devices.as_slice() {
        log::debug!("{}.{}.{}.: vend {:04x}, class {:06x}, head {:02x}", dev.bus(), dev.slot_fun().0, dev.slot_fun().1, dev.vendor_id(), dev.class_code().code(), dev.header_type());
    }

    let mmio_base = devices.as_slice().iter().find(|&dev| {
        dev.class_code().match_base_sub_interface(0x0c, 0x03, 0x30) && dev.vendor_id() == 0x8086
    }).and_then(|xdev| {
        log::debug!("An Intel xHC has been detected.");

        let switch_ehci_to_xhci = devices.as_slice().iter().find(|&dev| {
            dev.class_code().match_base_sub_interface(0x0c, 0x03, 0x20)
        }).is_some();

        if switch_ehci_to_xhci {
            // read and write `xdev`
            xdev.read_write_offset(0xdc, 0xd8); // Superspeed Ports
            xdev.read_write_offset(0xd4, 0xd0); // eHCi to xHCi ports
            log::debug!("Switched eHCi to xHCi.");
        }

        Some(xdev)
    }).or_else(|| {
        devices.as_slice().iter().find(|&dev| {
            dev.class_code().match_base_sub_interface(0x0c, 0x03, 0x30)
        })
    }).and_then(|xdev| {
        log::debug!("xHC has been found: {}.{}.{}.", xdev.bus(), xdev.slot_fun().0, xdev.slot_fun().1);

        xdev.bar0().mm()
    })?;

    Some(
        Controller::new(mmio_base, global_allocator())
    )
}

#[no_mangle]
pub extern "sysv64" fn _start (
    frame_buffer_info: FrameBuffer
) -> ! {
    // initialize globals
    globals::init(
        frame_buffer_info
    );

    log::info!("Hello, GYUR OS!");

    let mut screen_cell = globals::SCREEN.lock();
    let screen = screen_cell.get_mut().unwrap();

    screen.render_cursor();

    let mut xhc = setup_xhc_controller().unwrap();
    xhc.run();
    xhc.reconfigure_port();

    loop {
        xhc.process_events();
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