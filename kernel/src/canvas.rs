use crate::geometry::Disp2D;

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

/// A canvas interface, which is also called a `PixelWriter`.
pub trait Canvas {
    /// Returns the size of the canvas.
    fn size(&self) -> Disp2D;
    fn width(&self) -> isize {
        self.size().dx
    }
    fn height(&self) -> isize {
        self.size().dy
    }

    /// Write a color code into specific pixel.
    ///
    /// The `disp` parameter should be the displacement from the ltop of the desired canvas.
    fn render_pixel(&mut self, disp: Disp2D, c: ColorCode);
}