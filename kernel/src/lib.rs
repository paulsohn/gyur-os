#![no_std]
#![feature(
    abi_x86_interrupt,
    allocator_api,
    get_many_mut,
    lazy_cell,
    once_cell_try,
    const_nonnull_new,
    const_option,
    generic_arg_infer,
)]

mod sysfont;
pub mod geometry;
pub mod canvas;
pub mod screen;
pub mod console;
pub mod cursor;

pub mod pgmgr;
pub mod allocator;

pub mod pci;
pub mod xhci;
pub mod message;

pub mod window;

pub mod globals;