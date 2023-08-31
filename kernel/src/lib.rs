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
    base: *mut [u8; BYTES_PER_PIXEL],
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
            base: unsafe{ core::mem::transmute(fb_info.base) },
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
    fn write_pixel(&mut self, (x, y): (usize, usize), c: ColorCode) {
        debug_assert!(x < self.hor_res);
        debug_assert!(y < self.ver_res);

        let bytes = (self.formatter)(c);

        unsafe {
            let dst = self.base.add(self.stride * y + x);

            // volatile copy
            dst.write_volatile(bytes);
        }
        
    }

    pub fn write_rect(&mut self, (x, y): (usize, usize), (w, h): (usize, usize), c: ColorCode){
        for xx in x..x+w {
            for yy in y..y+h {
                self.write_pixel((xx,yy), c);
            }
        }
    }

    // @TODO : provide separate methods `write_ascii_bg` and `write_ascii_transparent`
    pub fn write_ascii(&mut self, (x, y): (usize, usize), ch: u8, fg: ColorCode, bg: Option<ColorCode>) {
        debug_assert!(ch <= 0x7f);

        use sysfont::SYSFONT;
        let bmp = &SYSFONT[ch as usize];

        for dy in 0..16usize {
            let row = bmp[dy];
            for dx in 0..8usize {
                if (row >> dx) & 1 != 0 {
                    self.write_pixel((x+dx,y+dy), fg);
                } else if let Some(bg) = bg {
                    self.write_pixel((x+dx,y+dy), bg);
                }
            }
        }
    }
}

const CONSOLE_ROWS: usize = 25;
const CONSOLE_COLS: usize = 80;

pub struct Console {
    screen: Screen,
    fg: ColorCode,
    bg: ColorCode,
    buffer: [[u8; CONSOLE_COLS]; CONSOLE_ROWS],
    // cur_row: usize, // fixed to CONSOLE_ROWS - 1
    cur_col: usize,

    // base_x: usize,
    // base_y: usize
}

impl Console {
    pub fn new(screen: Screen) -> Self {
        Self {
            screen,
            fg: ColorCode::BLACK,
            bg: ColorCode::WHITE,
            buffer: [[b' '; CONSOLE_COLS]; CONSOLE_ROWS],
            // cur_row: 0,
            cur_col: 0,
            // base_x: 0,
            //base_y : 0
        }
    }

    /// get the pixel coordinate from given buffer position (i,j).
    fn coord(&self, (i, j): (usize, usize)) -> (usize, usize){
        // (base_x+8*j,base_y+16*i)
        (8*j, 16*i)
    }

    /// refresh the screen by rendering chars in buffer.
    fn render(&mut self){
        for i in 0..CONSOLE_ROWS {
            for j in 0..CONSOLE_COLS {
                self.screen.write_ascii(self.coord((i,j)), self.buffer[i][j], self.fg, Some(self.bg));
            }
        }
    }

    /// rewind column position(carrige)
    /// this effectively mimics typewriter CR behavior.
    fn carrige_return(&mut self){
        self.cur_col = 0;
    }

    /// raise buffer contents by a line.
    /// this effectively mimics typewriter LF behavior, except rerendering.
    /// for newline behavior including rerendering, use `newline()` instead.
    fn line_feed(&mut self){
        for row in 1..CONSOLE_ROWS {
            self.buffer[row - 1] = self.buffer[row];
        }
        self.buffer[CONSOLE_ROWS-1] = [b' '; CONSOLE_COLS];
    }

    /// add new line.
    pub fn newline(&mut self){
        self.carrige_return();
        self.line_feed();
        self.render();
    }

    pub fn write_ascii(&mut self, ch: u8){
        // debug_assert!(i < CONSOLE_ROWS);
        // debug_assert!(j < CONSOLE_COLS);
        // debug_assert!(ch <= 0x7f);

        match ch { // @todo : more control characters support
            b'\n' => self.newline(),
            ch => {
                if self.cur_col >= CONSOLE_COLS {
                    self.newline();
                }

                let i = CONSOLE_ROWS - 1;
                let j = self.cur_col;
                self.buffer[i][j] = ch;
                self.screen.write_ascii(self.coord((i,j)), ch, self.fg, Some(self.bg));

                self.cur_col += 1;
            }
        }
    }
}

impl core::fmt::Write for Console {
    // fn write_str(&mut self, (x, y): (usize, usize), str: &str, c: ColorCode) {
    //     for (i, ch) in str.bytes().enumerate() {
    //         self.write_ascii((x + 8 * i, y), ch, c);
    //     }
    // }
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for ch in s.bytes() {
            self.write_ascii(ch);
        }
        Ok(())
    }
}