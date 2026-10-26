//! Points and cubic Bézier segments.

use crate::rect::Rect;

/// A point in the plane.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// Horizontal coordinate.
    pub x: f64,
    /// Vertical coordinate.
    pub y: f64,
}

impl Point {
    /// The origin.
    pub const ORIGIN: Self = Self { x: 0.0, y: 0.0 };

    /// Creates a point.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Interpolates towards `other`.
    pub fn lerp(self, other: Point, t: f64) -> Point {
        Point {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    /// The squared distance to `other`.
    pub fn distance_squared(self, other: Point) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        dx * dx + dy * dy
    }

    /// The distance to `other`.
    pub fn distance(self, other: Point) -> f64 {
        self.distance_squared(other).sqrt()
    }

    /// The length of the vector.
    pub fn length(self) -> f64 {
        self.distance_squared(Self::ORIGIN).sqrt()
    }

    /// The unit vector in the same direction.
    pub fn normalized(self) -> Point {
        let length = self.length();
        if length <= f64::EPSILON {
            Self::ORIGIN
        } else {
            Point::new(self.x / length, self.y / length)
        }
    }

    /// Whether both coordinates are finite.
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

/// A cubic Bézier segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicSegment {
    /// Start point.
    pub p0: Point,
    /// First control point.
    pub p1: Point,
    /// Second control point.
    pub p2: Point,
    /// End point.
    pub p3: Point,
}

impl CubicSegment {
    /// Creates a segment.
    pub const fn new(p0: Point, p1: Point, p2: Point, p3: Point) -> Self {
        Self { p0, p1, p2, p3 }
    }

    /// Creates a straight segment.
    pub fn line(from: Point, to: Point) -> Self {
        Self {
            p0: from,
            p1: from,
            p2: to,
            p3: to,
        }
    }

    /// The point at parameter `t` in `[0, 1]`.
    pub fn point(&self, t: f64) -> Point {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        let a = u * u * u;
        let b = 3.0 * u * u * t;
        let c = 3.0 * u * t * t;
        let d = t * t * t;

        Point {
            x: a * self.p0.x + b * self.p1.x + c * self.p2.x + d * self.p3.x,
            y: a * self.p0.y + b * self.p1.y + c * self.p2.y + d * self.p3.y,
        }
    }

    /// The derivative at parameter `t`.
    pub fn tangent(&self, t: f64) -> Point {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        let a = 3.0 * u * u;
        let b = 6.0 * u * t;
        let c = 3.0 * t * t;

        Point {
            x: a * (self.p1.x - self.p0.x)
                + b * (self.p2.x - self.p1.x)
                + c * (self.p3.x - self.p2.x),
            y: a * (self.p1.y - self.p0.y)
                + b * (self.p2.y - self.p1.y)
                + c * (self.p3.y - self.p2.y),
        }
    }

    /// An approximation of the arc length of the segment.
    pub fn approx_length(&self, samples: usize) -> f64 {
        let samples = samples.max(2);
        let mut length = 0.0;
        let mut previous = self.point(0.0);
        for step in 1..=samples {
            let current = self.point(step as f64 / samples as f64);
            length += current.distance(previous);
            previous = current;
        }
        length
    }

    /// The segment with the opposite direction.
    pub fn reversed(&self) -> Self {
        Self {
            p0: self.p3,
            p1: self.p2,
            p2: self.p1,
            p3: self.p0,
        }
    }

    /// The bounding box of the segment.
    pub fn bounds(&self) -> Rect {
        let mut bounds = Rect::from_edges(
            self.p0.x.min(self.p1.x).min(self.p2.x).min(self.p3.x),
            self.p0.y.min(self.p1.y).min(self.p2.y).min(self.p3.y),
            self.p0.x.max(self.p1.x).max(self.p2.x).max(self.p3.x),
            self.p0.y.max(self.p1.y).max(self.p2.y).max(self.p3.y),
        );

        // The curve can bulge past its control points, so the extremes of
        // the derivative are checked as well.
        for t in [0.0, 0.5, 1.0] {
            let tangent = self.tangent(t);
            if tangent.x.abs() > f64::EPSILON {
                let t = -self.p0.x / tangent.x;
                if (0.0..=1.0).contains(&t) {
                    let point = self.point(t);
                    bounds.x = bounds.x.min(point.x);
                    bounds.width = bounds.width.max(point.x - bounds.x);
                }
            }
            if tangent.y.abs() > f64::EPSILON {
                let t = -self.p0.y / tangent.y;
                if (0.0..=1.0).contains(&t) {
                    let point = self.point(t);
                    bounds.y = bounds.y.min(point.y);
                    bounds.height = bounds.height.max(point.y - bounds.y);
                }
            }
        }

        bounds
    }
}
