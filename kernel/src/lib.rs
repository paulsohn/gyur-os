#![no_std]
#![feature(
    abi_x86_interrupt,
    allocator_api,
    pointer_byte_offsets,
    get_many_mut,
    lazy_cell,
    once_cell_try,
    const_maybe_uninit_zeroed,
    const_nonnull_new,
    const_option
)]

mod sysfont;
pub mod geometry;
pub mod screen;
pub mod console;
pub mod cursor;

pub mod pci;
pub mod xhci;

pub mod message;

pub mod allocator;

pub mod globals;