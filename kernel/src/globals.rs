// print, println macros: credit goes to https://os.phil-opp.com/vga-text-mode/

use shared::FrameBufferInfo;
use crate::screen::Screen;
use crate::console::Console;

use core::fmt::{Arguments, Write};
use core::cell::OnceCell;
use spin::Mutex; // `Mutex<OnceCell<T>>` mimics std `OnceLock`.

pub static SCREEN: Mutex<OnceCell<Screen>> = Mutex::new(OnceCell::new());
pub static CONSOLE: Mutex<OnceCell<Console>> = Mutex::new(OnceCell::new());

pub fn init_globals(
    frame_buffer_info: FrameBufferInfo
){
    SCREEN.lock().get_or_init(|| {
        Screen::from(frame_buffer_info)
    });
    CONSOLE.lock().get_or_init(|| {
        Console::new(&SCREEN) // by invoking `new()`, we also render an empty console rectangle.
    });
}

pub fn _console_print(args: Arguments){
    CONSOLE.lock().get_mut().unwrap().write_fmt(args).unwrap();
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