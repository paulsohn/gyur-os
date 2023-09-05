use shared::frame_buffer::{
    FrameBuffer,
    PixelBytes,
    PixelFormat
};
use crate::sysfont::{
    SYSFONT,
    SYSFONT_WIDTH_PX,
    SYSFONT_HEIGHT_PX,
};

/// Color code.
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

    pub const GRAY   : Self = Self { r:127, g:127, b:127 };
}

/// A screen model wrapping frame buffer and its info.
pub struct Screen {
    frame_buffer: FrameBuffer,
    formatter: fn(ColorCode) -> PixelBytes // this effectively mimics the 'virtual method' pattern in other OOP language.
}

impl From<FrameBuffer> for Screen {
    fn from(frame_buffer: FrameBuffer) -> Self {
        fn rgb_formatter(c: ColorCode) -> PixelBytes {
            [c.r, c.g, c.b, c.b]
        }
        
        fn bgr_formatter(c: ColorCode) -> PixelBytes {
            [c.b, c.g, c.r, c.r]
        }

        Self {
            frame_buffer,
            formatter: match frame_buffer.format {
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
    pub fn render_pixel(&mut self, (x, y): (usize, usize), c: ColorCode) {
        // debug_assert!(x < self.hor_res);
        // debug_assert!(y < self.ver_res);

        let bytes = (self.formatter)(c);
        self.frame_buffer[(x,y)].write(bytes);
    }

    pub fn fill_rect(&mut self, (x, y): (usize, usize), (w, h): (usize, usize), c: ColorCode){
        for xx in x..x+w {
            for yy in y..y+h {
                self.render_pixel((xx,yy), c);
            }
        }
    }

    #[inline]
    pub fn fill_screen(&mut self, c: ColorCode) {
        self.fill_rect((0,0),self.frame_buffer.resolution(), c);
    }
    
    pub fn render_ascii(&mut self, (x, y): (usize, usize), ch: u8, fg: ColorCode, bg: Option<ColorCode>) {
        // debug_assert!(ch <= 0x7f);

        let bmp = &SYSFONT[ch as usize];

        if let Some(bg) = bg {
            self.fill_rect((x,y),(SYSFONT_WIDTH_PX,SYSFONT_HEIGHT_PX), bg);
        }

        for dy in 0..SYSFONT_HEIGHT_PX {
            let row = bmp[dy];
            for dx in 0..SYSFONT_WIDTH_PX {
                if (row >> dx) & 1 != 0 {
                    self.render_pixel((x+dx,y+dy), fg);
                }
                // else if let Some(bg) = bg {
                //     self.write_pixel((x+dx,y+dy), bg);
                // }
            }
        }
    }
}

// @TODO remove unsafe impl
// unsafe impl Sync for Screen {}
unsafe impl Send for Screen {}