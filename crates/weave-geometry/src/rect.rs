//! Axis aligned rectangles.

use weaver_motion::{Animatable, ValueSnapshot};

/// An axis aligned rectangle, described by its origin and its size.
///
/// The rectangle is allowed to have a zero or negative size in intermediate
/// states; adapters decide how to deal with that. Geometry helpers that
/// need a valid rectangle use [`Rect::is_valid`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width of the rectangle.
    pub width: f64,
    /// Height of the rectangle.
    pub height: f64,
}

impl Rect {
    /// Creates a rectangle from an origin and a size.
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates a rectangle from its edges.
    pub fn from_edges(left: f64, top: f64, right: f64, bottom: f64) -> Self {
        Self::new(left, top, right - left, bottom - top)
    }

    /// Creates a rectangle centred on `center` with the given size.
    pub fn from_center(center_x: f64, center_y: f64, width: f64, height: f64) -> Self {
        Self::new(
            center_x - width / 2.0,
            center_y - height / 2.0,
            width,
            height,
        )
    }

    /// The smallest rectangle containing this one and `other`.
    pub fn union(&self, other: &Rect) -> Rect {
        let left = self.min_x().min(other.min_x());
        let top = self.min_y().min(other.min_y());
        let right = self.max_x().max(other.max_x());
        let bottom = self.max_y().max(other.max_y());
        Rect::from_edges(left, top, right, bottom)
    }

    /// The overlapping area of two rectangles, if any.
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let left = self.min_x().max(other.min_x());
        let top = self.min_y().max(other.min_y());
        let right = self.max_x().min(other.max_x());
        let bottom = self.max_y().min(other.max_y());

        if right <= left || bottom <= top {
            None
        } else {
            Some(Rect::from_edges(left, top, right, bottom))
        }
    }

    /// Whether two rectangles overlap.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.intersection(other).is_some()
    }

    /// Whether a point lies inside the rectangle.
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.min_x() && x <= self.max_x() && y >= self.min_y() && y <= self.max_y()
    }

    /// The left edge.
    pub fn min_x(&self) -> f64 {
        self.x
    }

    /// The top edge.
    pub fn min_y(&self) -> f64 {
        self.y
    }

    /// The right edge.
    pub fn max_x(&self) -> f64 {
        self.x + self.width
    }

    /// The bottom edge.
    pub fn max_y(&self) -> f64 {
        self.y + self.height
    }

    /// Horizontal centre.
    pub fn center_x(&self) -> f64 {
        self.x + self.width / 2.0
    }

    /// Vertical centre.
    pub fn center_y(&self) -> f64 {
        self.y + self.height / 2.0
    }

    /// The area of the rectangle.
    pub fn area(&self) -> f64 {
        self.width.max(0.0) * self.height.max(0.0)
    }

    /// Moves the rectangle.
    pub fn translated(&self, dx: f64, dy: f64) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.width, self.height)
    }

    /// Scales the rectangle around its origin.
    pub fn scaled(&self, sx: f64, sy: f64) -> Rect {
        Rect::new(self.x * sx, self.y * sy, self.width * sx, self.height * sy)
    }

    /// Whether the rectangle has a positive size and finite coordinates.
    pub fn is_valid(&self) -> bool {
        self.width > 0.0 && self.height > 0.0 && self.is_finite()
    }

    /// Whether every coordinate is finite.
    pub fn is_finite(&self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
    }
}

impl Animatable for Rect {
    fn lerp(a: Self, b: Self, t: f64) -> Self {
        Rect {
            x: a.x + (b.x - a.x) * t,
            y: a.y + (b.y - a.y) * t,
            width: a.width + (b.width - a.width) * t,
            height: a.height + (b.height - a.height) * t,
        }
    }

    fn add_scaled(self, other: Self, scale: f64) -> Self {
        Rect {
            x: self.x + other.x * scale,
            y: self.y + other.y * scale,
            width: self.width + other.width * scale,
            height: self.height + other.height * scale,
        }
    }

    fn scale(self, factor: f64) -> Self {
        Rect {
            x: self.x * factor,
            y: self.y * factor,
            width: self.width * factor,
            height: self.height * factor,
        }
    }

    fn zero() -> Self {
        Rect::default()
    }

    fn distance(a: Self, b: Self) -> f64 {
        let dx = (a.x - b.x).abs();
        let dy = (a.y - b.y).abs();
        let dw = (a.width - b.width).abs();
        let dh = (a.height - b.height).abs();
        dx.max(dy).max(dw).max(dh)
    }

    fn integrate(
        value: Self,
        velocity: Self,
        target: Self,
        config: &weaver_motion::SpringConfig,
        dt: f64,
    ) -> (Self, Self) {
        let (x, vx) = weaver_motion::solve(config, value.x, velocity.x, target.x, dt);
        let (y, vy) = weaver_motion::solve(config, value.y, velocity.y, target.y, dt);
        let (width, vw) = weaver_motion::solve(config, value.width, velocity.width, target.width, dt);
        let (height, vh) =
            weaver_motion::solve(config, value.height, velocity.height, target.height, dt);

        (
            Rect {
                x,
                y,
                width,
                height,
            },
            Rect {
                x: vx,
                y: vy,
                width: vw,
                height: vh,
            },
        )
    }

    fn snapshot(self) -> ValueSnapshot {
        ValueSnapshot::Quad(weaver_motion::Vec4::new(
            self.x,
            self.y,
            self.width,
            self.height,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_are_derived_from_the_origin_and_the_size() {
        let rect = Rect::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(rect.min_x(), 10.0);
        assert_eq!(rect.max_x(), 40.0);
        assert_eq!(rect.min_y(), 20.0);
        assert_eq!(rect.max_y(), 60.0);
        assert_eq!(rect.center_x(), 25.0);
        assert_eq!(rect.center_y(), 40.0);
        assert_eq!(rect.area(), 1200.0);
    }

    #[test]
    fn intersection_of_disjoint_rectangles_is_empty() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(20.0, 20.0, 10.0, 10.0);
        assert!(!a.intersects(&b));
        assert!(a.intersection(&b).is_none());
    }

    #[test]
    fn intersection_and_union_are_complementary() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 10.0, 10.0);
        let overlap = a.intersection(&b).unwrap();
        assert_eq!(overlap, Rect::new(5.0, 5.0, 5.0, 5.0));

        let total = a.union(&b);
        assert_eq!(total, Rect::new(0.0, 0.0, 15.0, 15.0));
    }

    #[test]
    fn contains_respects_the_edges() {
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(rect.contains(5.0, 5.0));
        assert!(rect.contains(0.0, 0.0));
        assert!(rect.contains(10.0, 10.0));
        assert!(!rect.contains(-0.1, 5.0));
    }

    #[test]
    fn lerp_moves_towards_the_target() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(10.0, 20.0, 30.0, 40.0);
        let middle = Animatable::lerp(a, b, 0.5);
        assert_eq!(middle, Rect::new(5.0, 10.0, 20.0, 25.0));
    }
}
