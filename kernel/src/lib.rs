#![no_std]
#![feature(allocator_api)]
#![feature(pointer_byte_offsets)]
#![feature(get_many_mut)]
// #![feature(maybe_uninit_uninit_array)]
// #![feature(const_maybe_uninit_uninit_array)]

mod sysfont;
pub mod screen;
pub mod console;
pub mod cursor;

pub mod pci;
pub mod allocator;

pub enum Error {
    Full,
    Empty
}
pub type Result<T = ()> = core::result::Result<T, Error>;

pub mod globals;