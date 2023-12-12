// print, println macros: credit goes to https://os.phil-opp.com/vga-text-mode/

use crate::console::Console;

use core::fmt::{Arguments, Write};
use core::cell::OnceCell;
use spin::mutex::Mutex;

use x86_64::instructions::interrupts::without_interrupts;

pub static CONSOLE: Mutex<OnceCell<Console>> = Mutex::new(OnceCell::new());

/// Init [`CONSOLE`].
/// Should be called after initializing [`SCREEN`].
pub fn init(){
    CONSOLE.lock().get_or_init(|| {
        Console::new(&crate::globals::SCREEN) // by invoking `new()`, we also render an empty console rectangle.
    });
}

pub fn _console_print(args: Arguments){
    without_interrupts(|| {
        CONSOLE.lock().get_mut().unwrap().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! console_print {
    ($($arg:tt)*) => ($crate::globals::console::_console_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! console_println {
    () => ($crate::console_print!("\n"));
    ($($arg:tt)*) => ($crate::console_print!("{}\n", format_args!($($arg)*)));
}