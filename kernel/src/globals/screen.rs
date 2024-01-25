use crate::screen::Screen;
use shared::uefi_gop::{FrameBuffer, ModeInfo};

use core::cell::OnceCell;
use spin::mutex::Mutex;

pub static SCREEN: Mutex<OnceCell<Screen>> = Mutex::new(OnceCell::new());

pub fn init(frame_buffer: FrameBuffer<'static>, mode_info: ModeInfo) {
    SCREEN.lock().get_or_init(|| {
        Screen::new(frame_buffer, mode_info)
    });
}