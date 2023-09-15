#![no_std]

mod sysfont;
pub mod screen;
pub mod console;
pub mod cursor;

pub mod pci;
pub mod xhci_driver;

pub enum Error {
    Full,
    Empty
}
pub type Result<T = ()> = core::result::Result<T, Error>;

pub mod globals;