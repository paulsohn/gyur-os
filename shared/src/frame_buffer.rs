use core::ops::{Index, IndexMut};
// use volatile::Volatile; // crate volatile v0.3.0 for data-layer volatility

pub use uefi::proto::console::gop::{PixelFormat, ModeInfo};

pub const BYTES_PER_PIXEL: usize = 4;
pub type PixelBytes = [u8; BYTES_PER_PIXEL];

/// Raw information about frame buffer which is passed from bootloader to kernel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct FrameBuffer {
    base: *mut PixelBytes,
    stride: usize,
    /// horizontal pixel count.
    pub hor_res: usize,
    /// vertical pixel count.
    pub ver_res: usize,
    pub format: PixelFormat, // should not contain formatter itself
}

impl FrameBuffer {
    pub fn new(base: *mut u8, info: ModeInfo) -> Self {
        Self {
            base: unsafe{ core::mem::transmute(base) },
            stride: info.stride(),
            hor_res: info.resolution().0,
            ver_res: info.resolution().1,
            format: info.pixel_format()
        }
    }

    pub fn resolution(&self) -> (usize, usize) {
        (self.hor_res, self.ver_res)
    }
}

impl Index<(usize, usize)> for FrameBuffer {
    type Output = PixelBytes;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        unsafe { & *self.base.add(self.stride * y + x) }
    }
}

impl IndexMut<(usize, usize)> for FrameBuffer {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        unsafe { &mut *self.base.add(self.stride * y + x) }
    }
}