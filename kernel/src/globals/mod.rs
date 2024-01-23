
pub mod apic;

pub mod screen;
pub mod console;
pub mod logger;
pub mod xhci;
pub mod interrupts;

use shared::frame_buffer::FrameBuffer;

/// Initialize global variables.
pub fn init(
    frame_buffer: FrameBuffer
){
    screen::init(frame_buffer);
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