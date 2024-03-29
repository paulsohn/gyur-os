
pub mod apic;

pub mod screen;
pub mod console;
pub mod logger;

pub mod segments;
pub mod paging;
pub mod pgmgr;
pub mod allocator;

pub mod interrupts;
pub mod xhci;

pub mod message;

use shared::uefi_memory::MemoryMap;
use shared::KernelArgs;

/// Initialize global variables.
#[inline]
pub fn init(
    mmap: MemoryMap<'static>,
    args: KernelArgs
){
    // MMIO frame buffer and basic console, logging.
    screen::init(args.gop_frame_buffer, args.gop_mode_info);
    console::init(); // console depends on screen
    logger::init(); // logger depends on console

    // paging and memory.
    segments::init(); // load GDT and set segment registers.
    paging::init(); // load the identity(kernel) page table.
    pgmgr::init(&mmap);
    allocator::init(); // allocator depends on page manager.

    // interrupts and peripharals.
    interrupts::init(); // load IDT. actuall interrupts should occur AFTER xhci controller is set.
    xhci::init(); // xHCI depends on allocation

    x86_64::instructions::interrupts::enable();
}

pub use apic::APIC;

pub use screen::SCREEN;
pub use console::CONSOLE;
pub use xhci::XHC;

pub use message::MSG_QUEUE;