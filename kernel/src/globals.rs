// print, println macros: credit goes to https://os.phil-opp.com/vga-text-mode/

use shared::FrameBufferInfo;
use crate::screen::Screen;
use crate::console::Console;

use core::fmt::{Arguments, Write};
// use core::cell::OnceCell;
use spin::{Mutex, Once};

pub static SCREEN: Mutex<Once<Screen>> = Mutex::new(Once::new());
pub static CONSOLE: Mutex<Once<Console>> = Mutex::new(Once::new());

pub fn init_globals(
    frame_buffer_info: FrameBufferInfo
){
    SCREEN.lock().call_once(|| {
        Screen::from(frame_buffer_info)
    });
    CONSOLE.lock().call_once(|| {
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