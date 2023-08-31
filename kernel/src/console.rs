
use crate::screen::{
    ColorCode,
    Screen
};

const CONSOLE_ROWS: usize = 25;
const CONSOLE_COLS: usize = 80;

pub struct Console {
    screen: Screen, // @TODO screen should be owned or mut ref?
    fg: ColorCode,
    bg: ColorCode,
    buffer: [[u8; CONSOLE_COLS]; CONSOLE_ROWS],
    // cur_row: usize, // fixed to CONSOLE_ROWS - 1
    cur_col: usize,

    // base_x: usize,
    // base_y: usize
}

impl Console{
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
    fn screen_coord(&self, (i, j): (usize, usize)) -> (usize, usize){
        // (base_x+8*j,base_y+16*i)
        (8*j, 16*i)
    }

    /// refresh the screen by rendering chars in buffer.
    fn render(&mut self){
        for i in 0..CONSOLE_ROWS {
            for j in 0..CONSOLE_COLS {
                self.screen.write_ascii(self.screen_coord((i,j)), self.buffer[i][j], self.fg, Some(self.bg));
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
                if ch != self.buffer[i][j] { // reduce render process, especially for whitespaces
                    self.buffer[i][j] = ch;
                    self.screen.write_ascii(self.screen_coord((i,j)), ch, self.fg, Some(self.bg));
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