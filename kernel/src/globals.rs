// print, println macros: credit goes to https://os.phil-opp.com/vga-text-mode/

use shared::frame_buffer::FrameBuffer;
use crate::screen::Screen;
use crate::console::Console;

use core::fmt::{Arguments, Write};
use core::cell::OnceCell;
use spin::mutex::Mutex;

// `Mutex<OnceCell<T>>` mimics `std::sync::OnceLock`.
// This is not true if we replace `Mutex` into `RwLock`, since `OnceCell` do not implement `Sync` trait (the immutable method `.get_or_init()` on `OnceCell` do not provide actual sync between threads.)
// Simple check-skipping `Mutex<T>` or `RwLock<T>` can't be used here, since dummy value `unsafe { MaybeUninit::uninit().assume_init() }` (or even `zeroed()` instead of `uninit()`) are not permitted as a value of `T` and causes compile error, if `T` contains any reference values.

pub static SCREEN: Mutex<OnceCell<Screen>> = Mutex::new(OnceCell::new());
pub static CONSOLE: Mutex<OnceCell<Console>> = Mutex::new(OnceCell::new());

/// Initialize global variables.
pub fn init(
    frame_buffer: FrameBuffer
){
    // we need dynamic parameters for initialization, so we can't use `LazyCell` (or `LazyLock`)

    SCREEN.lock().get_or_init(|| {
        Screen::from(frame_buffer)
    });
    CONSOLE.lock().get_or_init(|| {
        Console::new(&SCREEN) // by invoking `new()`, we also render an empty console rectangle.
    });

    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LOGGER.filter);
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

/// The console logger.

pub struct Logger {
    pub filter: log::LevelFilter,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.filter
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            console_println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) { }
}

static LOGGER: Logger = Logger { filter: log::LevelFilter::Info };