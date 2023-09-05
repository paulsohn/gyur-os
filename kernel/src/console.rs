
use crate::screen::{
    ColorCode,
    Screen
};

use core::cell::OnceCell;
use spin::Mutex;

const CONSOLE_ROWS: usize = 25;
const CONSOLE_COLS: usize = 80;

const FONT_WIDTH_PX: usize = 8;
const FONT_HEIGHT_PX: usize = 16;

pub struct Console {
    screen: &'static Mutex<OnceCell<Screen>>, // note: methods accessing `screen` should be limited to `render()` and `render_one()`, to avoid requiring lock twice ("self-deadlock")

    fg: ColorCode,
    bg: ColorCode,
    buffer: [[u8; CONSOLE_COLS]; CONSOLE_ROWS],
    // cur_row: usize, // fixed to CONSOLE_ROWS - 1
    cur_col: usize,

    // base_x: usize,
    // base_y: usize
}

impl Console{
    pub fn new(screen: &'static Mutex<OnceCell<Screen>>) -> Self {
        let mut console = Self {
            screen,
            fg: ColorCode::WHITE,
            bg: ColorCode::GRAY,
            buffer: [[b' '; CONSOLE_COLS]; CONSOLE_ROWS],
            // cur_row: 0,
            cur_col: 0,
            // base_x: 0,
            //base_y : 0
        };
        console.render(); // initial rendering
        console
    }

    /// get the pixel coordinate from given buffer position (i,j).
    fn screen_coord(&self, (i, j): (usize, usize)) -> (usize, usize){
        // (base_x+8*j,base_y+16*i)
        (FONT_WIDTH_PX*j, FONT_HEIGHT_PX*i)
    }

    /// refresh the screen by rendering chars in buffer.
    fn render(&mut self){
        // to prevent E0716 (temporary value dropped while borrowed)
        // we should acquire the lock each time and save into a variable
        // to guarantee the lock is freed after we've finished using the underlying data

        let mut screen_lock = self.screen.lock();
        let screen = screen_lock.get_mut().unwrap();

        for i in 0..CONSOLE_ROWS {
            for j in 0..CONSOLE_COLS {
                screen.render_ascii(self.screen_coord((i,j)), self.buffer[i][j], self.fg, Some(self.bg));
            }
        }
    }

    /// refresh certain coordinate with given character.
    fn render_one(&mut self, (i, j): (usize, usize), ch: u8){
        let mut screen_lock = self.screen.lock();
        let screen = screen_lock.get_mut().unwrap();

        screen.render_ascii(self.screen_coord((i,j)), ch, self.fg, Some(self.bg));
    }

    /// Rewind column position(carrige).
    /// This effectively mimics typewriter CR behavior.
    #[inline]
    fn carrige_return(&mut self){
        self.cur_col = 0;
    }

    /// Raise buffer contents by a line.
    /// This effectively mimics typewriter LF behavior, except for rerendering.
    /// for unix-like newline behavior including rerendering, use `newline()` instead.
    #[inline]
    fn line_feed(&mut self){
        self.buffer.copy_within(1.., 0);
        self.buffer[CONSOLE_ROWS-1].fill(b' ');
        // self.buffer.last_mut().unwrap().fill(b' ');
        // self.buffer[CONSOLE_ROWS-1] = [b' '; CONSOLE_COLS];
    }

    /// Add new line.
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
                if ch != self.buffer[i][j] { // reduce render processes, especially for whitespaces
                    self.buffer[i][j] = ch;
                    self.render_one((i,j), ch);
                }

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
        s.bytes().for_each(|ch| { self.write_ascii(ch); });
        Ok(())
    }
}