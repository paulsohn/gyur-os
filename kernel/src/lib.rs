#![no_std]
#![feature(
    allocator_api,
    pointer_byte_offsets,
    get_many_mut,
    abi_x86_interrupt,
    lazy_cell,
    once_cell_try,
)]

mod sysfont;
pub mod screen;
pub mod console;
pub mod cursor;

pub mod pci;
pub mod xhci;
pub mod allocator;

pub mod globals;