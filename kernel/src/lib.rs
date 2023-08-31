#![no_std]

pub mod sysfont;


use shared::{
    FrameBufferInfo,
    PixelFormat,
    BYTES_PER_PIXEL
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColorCode {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl ColorCode {
    /// Construct a color code with given RGB.
    pub fn rgb(r: u8, g: u8, b:u8) -> Self {
        Self { r, g, b }
    }

    pub const BLACK  : Self = Self { r:0, g:0, b:0 };
    pub const RED    : Self = Self { r:255, g:0, b:0 };
    pub const GREEN  : Self = Self { r:0, g:255, b:0 };
    pub const BLUE   : Self = Self { r:0, g:0, b:255 };
    pub const CYAN   : Self = Self { r:0, g:255, b:255 };
    pub const MAGENTA: Self = Self { r:255, g:0, b:255 };
    pub const YELLOW : Self = Self { r:255, g:255, b:0 };
    pub const WHITE  : Self = Self { r:255, g:255, b:255 };
}

/// A screen model wrapping frame buffer and its info.
pub struct Screen {
    base: *mut u8,
    stride: usize,
    /// horizontal pixel count.
    pub hor_res: usize,
    /// vertical pixel count.
    pub ver_res: usize,
    formatter: fn(ColorCode) -> [u8; BYTES_PER_PIXEL] // this effectively mimics the 'virtual method' pattern in other OOP language.

    // @TODO instead of having a single formatter per screen,
    // how about implementing `Index<(usize, usize)>` (and `IndexMut` variant) with `Item = Pixel`
    // and each `Pixel` have its formatter?
    // (It might copy formatter address per pixel, which leads to a performance issue, but at least logically this make sense.)
}

fn rgb_formatter(c: ColorCode) -> [u8; BYTES_PER_PIXEL] {
    [c.r, c.g, c.b, c.b]
}

fn bgr_formatter(c: ColorCode) -> [u8; BYTES_PER_PIXEL] {
    [c.b, c.g, c.r, c.r]
}

impl From<FrameBufferInfo> for Screen {
    fn from(fb_info: FrameBufferInfo) -> Self {
        Self {
            base: fb_info.base,
            stride: fb_info.stride,
            hor_res: fb_info.hor_res,
            ver_res: fb_info.ver_res,
            formatter: match fb_info.format {
                // if this fourth-byte initialization gives extra overhead, we might try `MaybeUninit`.
                // Closure seems buggy here. Select normal functions here instead.
                PixelFormat::Rgb => rgb_formatter,
                PixelFormat::Bgr => bgr_formatter,
                _ => unimplemented!("Unsupported pixel format."),
            }
        }
    }
}

impl Screen {
    /// write a color code into specific pixel.
    pub fn write_pixel(&mut self, (x, y): (usize, usize), c: ColorCode) {
        debug_assert!(x < self.hor_res);
        debug_assert!(y < self.ver_res);

        let bytes = (self.formatter)(c);

        unsafe {
            let dst = self.base.add(BYTES_PER_PIXEL * (self.stride * y + x));

            // volatile copy
            // @TODO can we try 'volatile copy' here? (commented out)

            // let mut dst_slice = volatile::VolatilePtr::new(
            //     core::ptr::NonNull::from(
            //         core::slice::from_raw_parts_mut(dst, BYTES_PER_PIXEL)
            //     )
            // );
            // dst_slice.copy_from_slice(&bytes[..]);

            // core::intrinsics::volatile_copy_nonoverlapping_memory(dst, bytes.as_ptr(), bytes.len());

            dst.write_volatile(bytes[0]);
            dst.add(1).write_volatile(bytes[1]);
            dst.add(2).write_volatile(bytes[2]);
            // dst.add(3).write_volatile(bytes[3]);
        }
        
    }

    pub fn write_ascii(&mut self, (x, y): (usize, usize), ch: u8, c: ColorCode) {
        debug_assert!(ch <= 0x7f);

        use sysfont::SYSFONT;
        let bmp = &SYSFONT[ch as usize];

        for dy in 0..16usize {
            let row = bmp[dy];
            for dx in 0..8usize {
                if (row >> dx) & 1 != 0{
                    self.write_pixel((x+dx,y+dy), c);
                }
            }
        }
    }
}