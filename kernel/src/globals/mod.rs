
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
    xhci::init();

    interrupts::init();
}

pub use apic::APIC;

pub use screen::SCREEN;
pub use console::CONSOLE;
pub use xhci::XHC;