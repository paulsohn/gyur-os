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

    width: usize,
    height: usize,

    data: Vec<Vec<ColorCode>>,
}

impl Canvas for Window {
    fn size(&self) -> Pos2D {
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
    pub fn new(size: Pos2D) -> Self {
        // The default transparent color.
        let default_bg = ColorCode::BLACK;

        // The empty 2D vector filled with transparent color.
        // requires dynamic allocation.
        let mut data = Vec::with_capacity(size.y);
        data.resize_with(size.y, || {
            let mut row = Vec::with_capacity(size.x);
            row.resize(size.x, default_bg);
            row
        });

        Self {
            bg: Some(default_bg),

            width: size.x,
            height: size.y,

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
        &self.data[index.y][index.x]
    }
}
impl IndexMut<Pos2D> for Window {
    fn index_mut(&mut self, index: Pos2D) -> &mut Self::Output {
        // assert!(index.x < self.width);
        // assert!(index.y < self.height);
        &mut self.data[index.y][index.x]
    }
}