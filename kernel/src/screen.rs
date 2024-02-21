use crate::geometry::{
    Pos2D, Rect2D, Disp2D,
};

use crate::canvas::{
    ColorCode,
    Canvas,
};

use crate::sysfont::{
    SYSFONT,
    SYSFONT_WIDTH_PX,
    SYSFONT_HEIGHT_PX,
};
use crate::cursor::{
    SYSCURSOR_WIDTH_PX,
    SYSCURSOR_HEIGHT_PX,
    SYSCURSOR_SHAPE
};

use shared::uefi_gop::{
    FrameBuffer,
    ModeInfo,
    PixelFormat
};

pub const BYTES_PER_PIXEL: usize = 4;
pub type PixelBytes = [u8; BYTES_PER_PIXEL];
pub trait Formatter {
    fn write(&self, bytes: *mut PixelBytes, c: ColorCode);
}

static RGB_FORMATTER: RGBFormatter = RGBFormatter;
pub struct RGBFormatter;
impl Formatter for RGBFormatter {
    fn write(&self, bytes: *mut PixelBytes, c: ColorCode) {
        unsafe {
            bytes.write_volatile([c.r, c.g, c.b, 0]);
        }
    }
}

static BGR_FORMATTER: BGRFormatter = BGRFormatter;
pub struct BGRFormatter;
impl Formatter for BGRFormatter {
    fn write(&self, bytes: *mut PixelBytes, c: ColorCode) {
        unsafe {
            bytes.write_volatile([c.b, c.g, c.r, 0]);
        };
    }
}

/// A screen model wrapping frame buffer and its info.
pub struct Screen {
    /// The frame buffer base pointer.
    base: *mut u8,
    /// The frame buffer size.
    size: usize,

    /// Horizontal (actual) pixel count.
    stride: isize,
    /// Horizontal (displayed) pixel count.
    hor_res: isize,
    /// Vertical (displayed) pixel count.
    ver_res: isize,

    formatter: &'static dyn Formatter, // this effectively mimics the 'virtual method' pattern in other OOP language.

    cursor: Pos2D,
}

impl Canvas for Screen {
    fn size(&self) -> Disp2D {
        (self.hor_res, self.ver_res).into()
    }

    fn render_pixel(&mut self, pos: Pos2D, c: ColorCode) {
        let bytes = unsafe {
            self.base.cast::<PixelBytes>().offset(self.stride * pos.y + pos.x)
        };

        self.formatter.write(bytes, c);
    }
}

impl Screen {
    pub fn new(mut frame_buffer: FrameBuffer<'static>, mode_info: ModeInfo) -> Self {
        Self {
            base: frame_buffer.as_mut_ptr(),
            size: frame_buffer.size(),
            stride: mode_info.stride() as isize,
            hor_res: mode_info.resolution().0 as isize,
            ver_res: mode_info.resolution().1 as isize,

            formatter: match mode_info.pixel_format() {
                // `MaybeUninit` should not initialize fourth-byte.
                // Closure seems buggy here. Select normal functions here instead.
                PixelFormat::Rgb => &RGB_FORMATTER,
                PixelFormat::Bgr => &BGR_FORMATTER,
                _ => unimplemented!("Unsupported pixel format."),
            },

            cursor: (0, 0).into(),
        }
    }

    fn boundary(&self) -> Rect2D {
        Rect2D::from_points(
            Pos2D::ORIGIN,
            Pos2D::ORIGIN + self.size()
        )
    }

    pub fn fill_rect(&mut self, rect: Rect2D, c: ColorCode){
        rect.iterate_abs(|pos| {
            self.render_pixel(pos, c);
        });
    }

    #[inline]
    pub fn fill_screen(&mut self, c: ColorCode) {
        self.fill_rect(self.boundary(), c);
    }
    
    pub fn render_ascii(&mut self, ltop: Pos2D, ch: u8, fg: ColorCode, bg: Option<ColorCode>) {
        // debug_assert!(ch <= 0x7f);

        let rect = Rect2D::from_points(
            ltop,
            ltop + (SYSFONT_WIDTH_PX as isize, SYSFONT_HEIGHT_PX as isize).into()
        ).bound(self.boundary());

        let bmp = &SYSFONT[ch as usize];

        if let Some(bg) = bg {
            self.fill_rect(rect, bg);
        }

        rect.iterate_disp(|disp| {
            let row = bmp[disp.dy as usize];
            if (row >> disp.dx) & 1 != 0 {
                self.render_pixel(ltop + disp, fg);
            }
        });
    }
}

impl Screen {
    fn get_cursor_rect(&self) -> Rect2D {
        Rect2D::from_points(
            self.cursor,
            self.cursor + (SYSCURSOR_WIDTH_PX as isize, SYSCURSOR_HEIGHT_PX as isize).into()
        ).bound(self.boundary())
    }

    pub fn render_cursor(&mut self) {
        self.get_cursor_rect().iterate_disp(|disp| {
            let ch = match SYSCURSOR_SHAPE[disp.dy as usize][disp.dx as usize] {
                b'@' => ColorCode::BLACK,
                b'.' => ColorCode::WHITE,
                _ => return, // transparent
            };
            self.render_pixel(self.cursor + disp, ch);
        });
    }

    pub fn move_cursor(&mut self, disp: Disp2D) {
        self.fill_rect(
            self.get_cursor_rect(),
            ColorCode::GRAY // Console bg
        );
        self.cursor += disp;
        self.render_cursor();
    }
}

// unsafe impl Sync for Screen {}
unsafe impl Send for Screen {}