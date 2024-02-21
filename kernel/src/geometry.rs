use core::ops::{Add, AddAssign, Sub, Range};

/// A struct for screen coordinate position.
/// 
/// The origin is the screen lefttop corner, so this struct is intended to be used only by screen and window(layer) manager objects.
///
/// Ordinary canvases should use `Disp2D` instead.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Pos2D {
    pub x: isize,
    pub y: isize,
}

impl Pos2D {
    pub const ORIGIN: Self = Self { x: 0, y: 0 };

    fn clamp(&self, ltop: Pos2D, rbot: Pos2D) -> Pos2D {
        Pos2D {
            x: self.x.clamp(ltop.x, rbot.x),
            y: self.y.clamp(ltop.y, rbot.y)
        }
    }
}

impl From<(isize, isize)> for Pos2D {
    fn from((x, y): (isize, isize)) -> Self {
        Pos2D { x, y }
    }
}

impl From<Pos2D> for (isize, isize) {
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
        (self.x + rhs.dx, self.y + rhs.dy).into()
    }
}

impl Sub<Pos2D> for Pos2D {
    type Output = Disp2D;

    fn sub(self, rhs: Pos2D) -> Self::Output {
        (self.x - rhs.x, self.y - rhs.y).into()
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

impl AddAssign<Disp2D> for Disp2D {
    fn add_assign(&mut self, rhs: Disp2D) {
        *self = *self + rhs;
    }
}

impl Add<Disp2D> for Disp2D {
    type Output = Disp2D;

    fn add(self, rhs: Disp2D) -> Self::Output {
        (self.dx + rhs.dx, self.dy + rhs.dy).into()
    }
}

impl Disp2D {
    /// width of the displacement
    pub fn width(&self) -> isize {
        self.dx.abs()
    }

    /// height of the displacement
    pub fn height(&self) -> isize {
        self.dy.abs()
    }
}

/// A struct for a rectangular area.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rect2D {
    ltop: Pos2D,
    rbot: Pos2D,
}

impl Rect2D {
    /// The size of the rectangle, as the displacement vector of the major diagonal
    pub fn size(&self) -> Disp2D {
        self.rbot - self.ltop
    }

    pub fn width(&self) -> isize {
        self.size().dx
    }

    pub fn height(&self) -> isize {
        self.size().dy
    }

    pub fn from_points(p1: Pos2D, p2: Pos2D) -> Self {
        Self {
            ltop: (p1.x.min(p2.x), p1.y.min(p2.y)).into(),
            rbot: (p1.x.max(p2.x), p1.y.max(p2.y)).into()
        }
    }

    pub fn from_ranges(x_range: Range<isize>, y_range: Range<isize>) -> Self {
        Self::from_points(
            (x_range.start, y_range.start).into(),
            (x_range.end, y_range.end).into()
        )
    }

    pub fn bound(&self, boundary: Self) -> Self {
        Self::from_points(
            self.ltop.clamp(boundary.ltop, boundary.rbot),
            self.rbot.clamp(boundary.ltop, boundary.rbot),
        )
    }

    // pub fn iterate_abs<F: FnMut(Pos2D)>(&self, mut f: F) {
    //     for x in self.ltop.x .. self.rbot.x {
    //         for y in self.ltop.y .. self.rbot.y {
    //             f((x,y).into());
    //         }
    //     }
    // }

    pub fn iterate_disp<F: FnMut(Disp2D)>(&self, mut f: F) {
        let diag = self.rbot - self.ltop;
        for dx in 0 .. diag.dx {
            for dy in 0 .. diag.dy {
                f((dx,dy).into());
            }
        }
    }

    pub fn iterate_disp_bounded<F: FnMut(Disp2D)>(&self, boundary: Rect2D, mut f: F) {
        let bounded = self.bound(boundary);
        let ltop_rel = bounded.ltop - self.ltop;
        let rbot_rel = bounded.rbot - self.ltop;

        for dx in ltop_rel.dx .. rbot_rel.dx {
            for dy in ltop_rel.dy .. rbot_rel.dy {
                f((dx, dy).into());
            }
        }
    }
}