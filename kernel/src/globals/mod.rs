
pub mod apic;

pub mod screen;
pub mod console;
pub mod logger;
pub mod xhci;
pub mod interrupts;
pub mod message;

use shared::KernelArgs;

/// Initialize global variables.
pub fn init(args: KernelArgs){
    screen::init(args.gop_frame_buffer, args.gop_mode_info);
    console::init();
    logger::init();

    interrupts::init(); // load IDT first. actuall interrupts should occur AFTER xhci controller is set.
    xhci::init();

    x86_64::instructions::interrupts::enable();
}

pub use apic::APIC;

pub use screen::SCREEN;
pub use console::CONSOLE;
pub use xhci::XHC;

pub use message::MSG_QUEUE;