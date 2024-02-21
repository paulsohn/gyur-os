extern crate alloc;

use crate::geometry::{Disp2D, Rect2D};
use crate::canvas::{ColorCode, Canvas};
// use crate::screen::Screen;

use core::ops::{Index, IndexMut};

use alloc::vec::Vec;

// use spin::mutex::Mutex;

pub struct Window {
    // screen: &'static Mutex<OnceCell<Screen>>,

    /// transparent color. `None` if every color code is valid.
    bg: Option<ColorCode>,

    rect: Rect2D,

    data: Vec<Vec<ColorCode>>,
}

impl Canvas for Window {
    fn size(&self) -> Disp2D {
        self.rect.size()
    }

    fn render_pixel(&mut self, disp: Disp2D, c: ColorCode) {
        self[disp] = c;
    }
}

impl Window {
    /// Creates a new window with the given size.
    /// 
    /// Requires dynamic allocation.
    pub fn new(rect: Rect2D) -> Self {

        let width = rect.width() as usize;
        let height = rect.height() as usize;

        // The default transparent color.
        let default_bg = ColorCode::BLACK;

        // The empty 2D vector filled with transparent color.
        // requires dynamic allocation.
        let mut data = Vec::with_capacity(height);
        data.resize_with(height, || {
            let mut row = Vec::with_capacity(width);
            row.resize(width, default_bg);
            row
        });

        Self {
            bg: Some(default_bg),
            rect,
            data
        }
    }

    /// Set the background color of this window.
    pub fn set_bg(&mut self, bg: Option<ColorCode>) {
        self.bg = bg;
    }
}

impl Index<Disp2D> for Window {
    type Output = ColorCode;

    fn index(&self, index: Disp2D) -> &Self::Output {
        // assert!(index.dx < self.rect.width());
        // assert!(index.dy < self.rect.height());
        let i = index.dy as usize;
        let j = index.dx as usize;
        &self.data[i][j]
    }
}
impl IndexMut<Disp2D> for Window {
    fn index_mut(&mut self, index: Disp2D) -> &mut Self::Output {
        // assert!(index.dx < self.rect.width());
        // assert!(index.dy < self.rect.height());
        let i = index.dy as usize;
        let j = index.dx as usize;
        &mut self.data[i][j]
    }
}