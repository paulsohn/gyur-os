extern crate alloc;

use crate::geometry::{Disp2D, Pos2D, Rect2D};
use crate::canvas::{ColorCode, Canvas};
// use crate::screen::Screen;

use core::ops::{Index, IndexMut};

use alloc::vec::Vec;

// use spin::mutex::Mutex;

pub struct Window {
    // screen: &'static Mutex<OnceCell<Screen>>,

    /// transparent color. `None` if every color code is valid.
    bg: Option<ColorCode>,

    width: isize,
    height: isize,

    data: Vec<Vec<ColorCode>>,
}

impl Canvas for Window {
    fn size(&self) -> Disp2D {
        (self.width, self.height).into()
    }

    fn render_pixel(&mut self, pos: Pos2D, c: ColorCode) {
        self[pos] = c;
    }
}

impl Window {
    /// Creates a new window with the given size.
    /// 
    /// Requires dynamic allocation.
    pub fn new(size: Disp2D) -> Self {

        let width = size.width();
        let height = size.height();

        // The default transparent color.
        let default_bg = ColorCode::BLACK;

        // The empty 2D vector filled with transparent color.
        // requires dynamic allocation.
        let mut data = Vec::with_capacity(height as usize);
        data.resize_with(height as usize, || {
            let mut row = Vec::with_capacity(width as usize);
            row.resize(width as usize, default_bg);
            row
        });

        Self {
            bg: Some(default_bg),

            width,
            height,

            data
        }
    }

    /// Set the background color of this window.
    pub fn set_bg(&mut self, bg: Option<ColorCode>) {
        self.bg = bg;
    }
}

impl Index<Pos2D> for Window {
    type Output = ColorCode;

    fn index(&self, index: Pos2D) -> &Self::Output {
        // assert!(index.x < self.width);
        // assert!(index.y < self.height);
        &self.data[index.i()][index.j()]
    }
}
impl IndexMut<Pos2D> for Window {
    fn index_mut(&mut self, index: Pos2D) -> &mut Self::Output {
        // assert!(index.x < self.width);
        // assert!(index.y < self.height);
        &mut self.data[index.i()][index.j()]
    }
}