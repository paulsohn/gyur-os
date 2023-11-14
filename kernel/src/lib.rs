#![no_std]

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