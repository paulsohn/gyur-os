use shared::frame_buffer::FrameBuffer;

use crate::screen::Screen;

use core::cell::OnceCell;
use spin::mutex::Mutex;

pub static SCREEN: Mutex<OnceCell<Screen>> = Mutex::new(OnceCell::new());

pub fn init(frame_buffer: FrameBuffer) {
    SCREEN.lock().get_or_init(|| {
        Screen::from(frame_buffer)
    });
}