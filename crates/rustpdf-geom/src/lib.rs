//! rustpdf-geom — 2D geometry primitives and page units for PDF processing.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A point in 2D space.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// An axis-aligned rectangle represented by coordinates [min_x, min_y, max_x, max_y].
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rect {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl Rect {
    pub const ZERO: Self = Self {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 0.0,
        max_y: 0.0,
    };

    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self {
            min_x: x0.min(x1),
            min_y: y0.min(y1),
            max_x: x0.max(x1),
            max_y: y0.max(y1),
        }
    }

    pub fn from_points(p0: Point, p1: Point) -> Self {
        Self::new(p0.x, p0.y, p1.x, p1.y)
    }

    pub fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(x, y, x + width, y + height)
    }

    pub fn width(self) -> f32 {
        (self.max_x - self.min_x).max(0.0)
    }

    pub fn height(self) -> f32 {
        (self.max_y - self.min_y).max(0.0)
    }

    pub fn contains(self, p: Point) -> bool {
        p.x >= self.min_x && p.x <= self.max_x && p.y >= self.min_y && p.y <= self.max_y
    }

    pub fn intersects(self, other: Self) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    pub fn intersect(self, other: Self) -> Option<Self> {
        let min_x = self.min_x.max(other.min_x);
        let min_y = self.min_y.max(other.min_y);
        let max_x = self.max_x.min(other.max_x);
        let max_y = self.max_y.min(other.max_y);
        if min_x <= max_x && min_y <= max_y {
            Some(Self {
                min_x,
                min_y,
                max_x,
                max_y,
            })
        } else {
            None
        }
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }

    pub fn as_array(self) -> [f32; 4] {
        [self.min_x, self.min_y, self.max_x, self.max_y]
    }
}

/// A 2D affine transformation matrix:
/// [ a  c  e ]
/// [ b  d  f ]
/// [ 0  0  1 ]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Transform {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        }
    }

    pub fn translate(tx: f32, ty: f32) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: tx,
            f: ty,
        }
    }

    pub fn rotate_rad(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }

    pub fn rotate_deg(degrees: f32) -> Self {
        Self::rotate_rad(degrees.to_radians())
    }

    pub fn then(self, other: Self) -> Self {
        Self {
            a: other.a * self.a + other.c * self.b,
            b: other.b * self.a + other.d * self.b,
            c: other.a * self.c + other.c * self.d,
            d: other.b * self.c + other.d * self.d,
            e: other.a * self.e + other.c * self.f + other.e,
            f: other.b * self.e + other.d * self.f + other.f,
        }
    }

    pub fn transform_point(self, p: Point) -> Point {
        Point {
            x: self.a * p.x + self.c * p.y + self.e,
            y: self.b * p.x + self.d * p.y + self.f,
        }
    }
}

/// PDF Unit conversions (1 inch = 72 PDF points = 25.4 millimeters).
pub struct PageUnits;

impl PageUnits {
    pub const POINTS_PER_INCH: f32 = 72.0;
    pub const MM_PER_INCH: f32 = 25.4;

    pub fn pt_to_mm(pt: f32) -> f32 {
        pt * (Self::MM_PER_INCH / Self::POINTS_PER_INCH)
    }

    pub fn mm_to_pt(mm: f32) -> f32 {
        mm * (Self::POINTS_PER_INCH / Self::MM_PER_INCH)
    }

    pub fn pt_to_in(pt: f32) -> f32 {
        pt / Self::POINTS_PER_INCH
    }

    pub fn in_to_pt(inches: f32) -> f32 {
        inches * Self::POINTS_PER_INCH
    }

    pub fn pt_to_px(pt: f32, dpi: f32) -> f32 {
        pt * (dpi / Self::POINTS_PER_INCH)
    }

    pub fn px_to_pt(px: f32, dpi: f32) -> f32 {
        px * (Self::POINTS_PER_INCH / dpi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_math() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = Rect::new(50.0, 50.0, 150.0, 150.0);
        assert_eq!(r1.width(), 100.0);
        assert_eq!(r1.height(), 100.0);
        assert!(r1.contains(Point::new(50.0, 50.0)));
        assert!(!r1.contains(Point::new(101.0, 50.0)));

        let inter = r1.intersect(r2).unwrap();
        assert_eq!(inter, Rect::new(50.0, 50.0, 100.0, 100.0));
    }

    #[test]
    fn test_transform() {
        let t = Transform::translate(10.0, 20.0).then(Transform::scale(2.0, 3.0));
        let p = t.transform_point(Point::new(5.0, 5.0));
        // (5+10)*2 = 30, (5+20)*3 = 75
        assert_eq!(p, Point::new(30.0, 75.0));
    }

    #[test]
    fn test_page_units() {
        assert!((PageUnits::in_to_pt(1.0) - 72.0).abs() < 1e-4);
        assert!((PageUnits::mm_to_pt(25.4) - 72.0).abs() < 1e-4);
    }
}
