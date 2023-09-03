
use crate::screen::{
    ColorCode,
    Screen
};

use core::ops::DerefMut;
// use core::cell::OnceCell;
use spin::{Mutex, Once};

const CONSOLE_ROWS: usize = 25;
const CONSOLE_COLS: usize = 80;

pub struct Console {
    screen: &'static Once<Mutex<Screen>>,

    fg: ColorCode,
    bg: ColorCode,
    buffer: [[u8; CONSOLE_COLS]; CONSOLE_ROWS],
    // cur_row: usize, // fixed to CONSOLE_ROWS - 1
    cur_col: usize,

    // base_x: usize,
    // base_y: usize
}

impl Console{
    pub fn new(screen: &'static Once<Mutex<Screen>>) -> Self {
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
        (8*j, 16*i)
    }

    /// refresh the screen by rendering chars in buffer.
    fn render(&mut self){
        // to prevent E0716 (temporary value dropped while borrowed)
        // we should acquire the lock each time and save into a variable
        // to guarantee the lock is freed after we've finished using the underlying data

        unsafe{ core::arch::asm!("mov r11, 0xCAFE1"); }

        let mut screen_lock = self.screen.get().unwrap().lock();
        let screen = screen_lock.deref_mut();

        // @TODO : seems that the lock is not properly released
        // stops here at 2nd iteration

        unsafe{ core::arch::asm!("mov r10, 0xCAFE2"); }

        for i in 0..CONSOLE_ROWS {
            for j in 0..CONSOLE_COLS {
                screen.write_ascii(self.screen_coord((i,j)), self.buffer[i][j], self.fg, Some(self.bg));
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

        let mut screen_lock = self.screen.get().unwrap().lock();
        let screen = screen_lock.deref_mut();

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

                    screen.write_ascii(self.screen_coord((i,j)), ch, self.fg, Some(self.bg));
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
        for ch in s.bytes() {
            self.write_ascii(ch);
        }
        Ok(())
    }
}

// @TODO remove unsafe impl
unsafe impl Sync for Console {}
unsafe impl Send for Console {}