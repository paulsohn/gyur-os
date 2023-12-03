use core::ops::{Add, AddAssign, Sub, Range};

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
use crate::cursor::{
    SYSCURSOR_WIDTH,
    SYSCURSOR_HEIGHT,
    SYSCURSOR_SHAPE
};

/// A struct for screen coordinate position.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Pos2D {
    pub x: usize,
    pub y: usize,
}

impl Pos2D {
    fn bound(&self, boundary: Pos2D) -> Pos2D {
        Pos2D {
            x: self.x.min(boundary.x),
            y: self.y.min(boundary.y)
        }
    }
}

impl From<(usize, usize)> for Pos2D {
    fn from((x, y): (usize, usize)) -> Self {
        assert!(x <= isize::MAX as usize);
        assert!(y <= isize::MAX as usize);
        Pos2D { x, y }
    }
}

impl From<Pos2D> for (usize, usize) {
    fn from(pos: Pos2D) -> Self {
        (pos.x, pos.y)
    }
}

impl AddAssign<Disp2D> for Pos2D {
    fn add_assign(&mut self, rhs: Disp2D) {
        *self = *self + rhs;
    }
}

impl Add<Disp2D> for Pos2D {
    type Output = Pos2D;

    fn add(self, rhs: Disp2D) -> Self::Output {
        // want to use the intrinsic `arith_offset` function.
        Pos2D::from((
            (self.x as isize).wrapping_add(rhs.dx).clamp(0, isize::MAX) as usize,
            (self.y as isize).wrapping_add(rhs.dy).clamp(0, isize::MAX) as usize
        ))
    }
}

impl Sub<Pos2D> for Pos2D {
    type Output = Disp2D;

    fn sub(self, rhs: Pos2D) -> Self::Output {
        Disp2D::from((
            (self.x as isize) - (rhs.x as isize),
            (self.y as isize) - (rhs.y as isize),
        ))
    }
}

/// A struct for screen coordinate displacement.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Disp2D {
    pub dx: isize,
    pub dy: isize,
}

impl From<(isize, isize)> for Disp2D {
    fn from((dx, dy): (isize, isize)) -> Self {
        Disp2D { dx, dy }
    }
}

/// A struct for screen rectangular area.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rect2D {
    pub left_top: Pos2D,
    pub diag: Disp2D,
}

impl Rect2D {
    pub fn from_lefttop_rightbot(left_top: Pos2D, right_bot: Pos2D) -> Self {
        Rect2D::from_lefttop_diag(left_top, right_bot - left_top)
    }

    pub fn from_lefttop_diag_boundary(left_top: Pos2D, diag: Disp2D, boundary: Pos2D) -> Self {
        let actual_diag = (left_top + diag).bound(boundary) - left_top;
        Rect2D::from_lefttop_diag(left_top, actual_diag)
    }

    pub fn from_lefttop_diag(left_top: Pos2D, diag: Disp2D) -> Self {
        debug_assert!(diag.dx >= 0);
        debug_assert!(diag.dy >= 0);
        Rect2D {
            left_top,
            diag,
        }
    }

    pub fn from_ranges(x_range: Range<usize>, y_range: Range<usize>) -> Self {
        Rect2D {
            left_top: (x_range.start, y_range.start).into(),
            diag: (x_range.len() as isize, y_range.len() as isize).into(),
        }
    }

    pub fn iterate_abs<F: FnMut(Pos2D)>(&self, mut f: F) {
        let left_top = self.left_top;
        let right_bot = self.left_top + self.diag;
        for x in left_top.x .. right_bot.x {
            for y in left_top.y .. right_bot.y {
                f((x,y).into());
            }
        }
    }

    pub fn iterate_disp<F: FnMut(Disp2D)>(&self, mut f: F) {
        for dx in 0..self.diag.dx {
            for dy in 0..self.diag.dy {
                f((dx,dy).into());
            }
        }
    }
}

/// Color code.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColorCode {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorCode {
    /// Construct a color code with given RGB.
    #[allow(invalid_value)]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const BLACK  : Self = Self::rgb(0, 0, 0);
    pub const RED    : Self = Self::rgb(255, 0, 0);
    pub const GREEN  : Self = Self::rgb(0, 255, 0);
    pub const BLUE   : Self = Self::rgb(0, 0, 255);
    pub const CYAN   : Self = Self::rgb(0, 255, 255);
    pub const MAGENTA: Self = Self::rgb(255, 0, 255);
    pub const YELLOW : Self = Self::rgb(255, 255, 0);
    pub const WHITE  : Self = Self::rgb(255, 255, 255);

    pub const GRAY   : Self = Self::rgb(127, 127, 127);
}

pub trait Formatter {
    fn write(&self, bytes: &mut PixelBytes, c: ColorCode);
}

static RGB_FORMATTER: RGBFormatter = RGBFormatter;
pub struct RGBFormatter;
impl Formatter for RGBFormatter {
    fn write(&self, bytes: &mut PixelBytes, c: ColorCode) {
        unsafe {
            let [r, g, b] = bytes.get_many_unchecked_mut([0, 1, 2]);

            (r as *mut u8).write_volatile(c.r);
            (g as *mut u8).write_volatile(c.g);
            (b as *mut u8).write_volatile(c.b);

            // (&mut bytes[0] as *mut u8).write_volatile(c.r);
            // (&mut bytes[1] as *mut u8).write_volatile(c.g);
            // (&mut bytes[2] as *mut u8).write_volatile(c.b);

            // (bytes as *mut PixelBytes).write_volatile([c.r, c.g, c.b, 0]);
        }

        // bytes[0] = c.r;
        // bytes[1] = c.g;
        // bytes[2] = c.b;
    }
}

static BGR_FORMATTER: BGRFormatter = BGRFormatter;
pub struct BGRFormatter;
impl Formatter for BGRFormatter {
    fn write(&self, bytes: &mut PixelBytes, c: ColorCode) {
        unsafe {
            let [r, g, b] = bytes.get_many_unchecked_mut([2, 1, 0]);

            (r as *mut u8).write_volatile(c.r);
            (g as *mut u8).write_volatile(c.g);
            (b as *mut u8).write_volatile(c.b);

            // (&mut bytes[0] as *mut u8).write_volatile(c.b);
            // (&mut bytes[1] as *mut u8).write_volatile(c.g);
            // (&mut bytes[2] as *mut u8).write_volatile(c.r);

            // (bytes as *mut PixelBytes).write_volatile([c.b, c.g, c.r, 0]);
        }

        // bytes[0] = c.b;
        // bytes[1] = c.g;
        // bytes[2] = c.r;
    }
}

/// A screen model wrapping frame buffer and its info.
pub struct Screen {
    frame_buffer: FrameBuffer,
    formatter: &'static dyn Formatter, // this effectively mimics the 'virtual method' pattern in other OOP language.

    cursor: Pos2D,
}

impl From<FrameBuffer> for Screen {
    fn from(frame_buffer: FrameBuffer) -> Self {
        Self {
            frame_buffer,
            formatter: match frame_buffer.format {
                // `MaybeUninit` should not initialize fourth-byte.
                // Closure seems buggy here. Select normal functions here instead.
                PixelFormat::Rgb => &RGB_FORMATTER,
                PixelFormat::Bgr => &BGR_FORMATTER,
                _ => unimplemented!("Unsupported pixel format."),
            },

            cursor: Pos2D::from((0, 0)),
        }
    }
}

impl Screen {
    #[inline]
    pub fn resolution(&self) -> Pos2D {
        self.frame_buffer.resolution().into()
    }

    /// write a color code into specific pixel.
    pub fn render_pixel(&mut self, pos: Pos2D, c: ColorCode) {
        self.formatter.write(&mut self.frame_buffer[pos.into()], c);
    }

    pub fn fill_rect(&mut self, rect: Rect2D, c: ColorCode){
        rect.iterate_abs(|pos| {
            self.render_pixel(pos, c);
        });
    }

    #[inline]
    pub fn fill_screen(&mut self, c: ColorCode) {
        self.fill_rect(
            Rect2D::from_lefttop_rightbot(
                (0,0).into(),
                self.resolution()
            ),
            c
        );
    }
    
    pub fn render_ascii(&mut self, left_top: Pos2D, ch: u8, fg: ColorCode, bg: Option<ColorCode>) {
        // debug_assert!(ch <= 0x7f);

        let rect = Rect2D::from_lefttop_diag_boundary(
            left_top,
            (SYSFONT_WIDTH_PX as isize, SYSFONT_HEIGHT_PX as isize).into(),
            self.resolution()
        );

        let bmp = &SYSFONT[ch as usize];

        if let Some(bg) = bg {
            self.fill_rect(rect, bg);
        }

        rect.iterate_disp(|disp| {
            let row = bmp[disp.dy as usize];
            if (row >> disp.dx) & 1 != 0 {
                self.render_pixel(left_top + disp, fg);
            }
        });
    }
}

impl Screen {
    fn get_cursor_rect(&self) -> Rect2D {
        Rect2D::from_lefttop_diag_boundary(
            self.cursor,
            (SYSCURSOR_WIDTH as isize, SYSCURSOR_HEIGHT as isize).into(),
            self.resolution(),
        )
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