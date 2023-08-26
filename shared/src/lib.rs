#![no_std]
extern crate uefi;

pub use uefi::proto::console::gop::{PixelFormat, ModeInfo};

pub const BYTES_PER_PIXEL: usize = 4;

/// Raw information about frame buffer which is passed from bootloader to kernel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct FrameBufferInfo {
    pub base: *mut u8,
    pub stride: usize,
    pub hor_res: usize,
    pub ver_res: usize,
    pub format: PixelFormat,
}

impl FrameBufferInfo {
    pub fn new(base: *mut u8, info: ModeInfo) -> Self {
        Self {
            base,
            stride: info.stride(),
            hor_res: info.resolution().0,
            ver_res: info.resolution().1,
            format: info.pixel_format()
        }
    }
}

// /// kernel argument type
// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
// #[repr(C)]
// pub struct KernelArgs {
//     pub frame_buffer_info: FrameBufferInfo
// }
