// print, println macros: credit goes to https://os.phil-opp.com/vga-text-mode/

use shared::FrameBufferInfo;
use crate::screen::Screen;
use crate::console::Console;

use core::fmt::{Arguments, Write};
// use core::cell::OnceCell;
use spin::{Mutex, Once};

pub static SCREEN: Once<Mutex<Screen>> = Once::new();
pub static CONSOLE: Once<Mutex<Console>> = Once::new();

pub fn init_globals(
    frame_buffer_info: FrameBufferInfo
){
    SCREEN.call_once(|| {
        Mutex::new(Screen::from(frame_buffer_info))
    });
    CONSOLE.call_once(|| {
        Mutex::new(Console::new(&SCREEN)) // by invoking `new()`, we also render an empty console rectangle.
    });
}

pub fn _console_print(args: Arguments){
    CONSOLE.get().unwrap().lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! console_print {
    ($($arg:tt)*) => ($crate::globals::_console_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! console_println {
    () => ($crate::console_print!("\n"));
    ($($arg:tt)*) => ($crate::console_print!("{}\n", format_args!($($arg)*)));
}