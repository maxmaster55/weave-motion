//! Contours and paths.

use alloc::vec::Vec;

use crate::point::{CubicSegment, Point};
use crate::rect::Rect;

/// A closed or open sequence of cubic Bézier segments.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contour {
    /// Whether the contour loops back to its start.
    pub closed: bool,
    /// The segments of the contour.
    pub segments: Vec<CubicSegment>,
}

impl Contour {
    /// Creates a contour from segments.
    pub fn new(closed: bool, segments: Vec<CubicSegment>) -> Self {
        Self { closed, segments }
    }

    /// Creates a contour from a polyline.
    ///
    /// The polyline is converted into cubic segments whose control points
    /// sit on the segment itself, so the shape is preserved exactly.
    pub fn from_points(points: &[Point], closed: bool) -> Self {
        let mut segments = Vec::new();

        if points.len() < 2 {
            return Self { closed, segments };
        }

        let count = if closed {
            points.len()
        } else {
            points.len() - 1
        };
        for index in 0..count {
            let from = points[index];
            let to = points[(index + 1) % points.len()];
            segments.push(CubicSegment::line(from, to));
        }

        Self { closed, segments }
    }

    /// The endpoints of the segments.
    pub fn points(&self) -> Vec<Point> {
        let mut points = Vec::with_capacity(self.segments.len());
        for segment in &self.segments {
            points.push(segment.p0);
        }
        points
    }

    /// The bounding box of the contour.
    pub fn bounds(&self) -> Rect {
        let mut bounds: Option<Rect> = None;
        for segment in &self.segments {
            let segment_bounds = segment.bounds();
            bounds = Some(match bounds {
                Some(current) => current.union(&segment_bounds),
                None => segment_bounds,
            });
        }
        bounds.unwrap_or_default()
    }

    /// The signed area of the contour.
    ///
    /// The sign tells the winding direction: positive is counter-clockwise
    /// in a coordinate system where `y` grows upwards.
    pub fn signed_area(&self) -> f64 {
        let points = self.points();
        if points.len() < 3 {
            return 0.0;
        }

        let mut area = 0.0;
        for index in 0..points.len() {
            let current = points[index];
            let next = points[(index + 1) % points.len()];
            area += current.x * next.y - next.x * current.y;
        }
        area / 2.0
    }

    /// Whether the contour is wound clockwise.
    pub fn is_clockwise(&self) -> bool {
        self.signed_area() < 0.0
    }

    /// The contour with the opposite winding direction.
    pub fn reversed(&self) -> Self {
        let mut segments: Vec<CubicSegment> =
            self.segments.iter().rev().map(|s| s.reversed()).collect();
        if !self.closed {
            segments.reverse();
        }
        Self {
            closed: self.closed,
            segments,
        }
    }

    /// The total arc length of the contour.
    pub fn approx_length(&self, samples: usize) -> f64 {
        self.segments
            .iter()
            .map(|segment| segment.approx_length(samples))
            .sum()
    }
}

/// A path: one or more contours.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path {
    /// The contours of the path.
    pub contours: Vec<Contour>,
}

impl Path {
    /// Creates a path from contours.
    pub fn new(contours: Vec<Contour>) -> Self {
        Self { contours }
    }

    /// Creates a path from a single polyline.
    pub fn from_points(points: &[Point], closed: bool) -> Self {
        Self {
            contours: alloc::vec![Contour::from_points(points, closed)],
        }
    }

    /// The bounding box of the whole path.
    pub fn bounds(&self) -> Rect {
        let mut bounds: Option<Rect> = None;
        for contour in &self.contours {
            let contour_bounds = contour.bounds();
            bounds = Some(match bounds {
                Some(current) => current.union(&contour_bounds),
                None => contour_bounds,
            });
        }
        bounds.unwrap_or_default()
    }

    /// Whether two paths can be morphed into each other.
    ///
    /// Two paths are compatible when they have the same number of contours
    /// and the same open/closed structure. Anything else would need a
    /// topology change, which Weave does not attempt: the caller has to
    /// fall back to a crossfade.
    pub fn is_compatible_with(&self, other: &Path) -> bool {
        if self.contours.len() != other.contours.len() {
            return false;
        }
        self.contours
            .iter()
            .zip(other.contours.iter())
            .all(|(a, b)| a.closed == b.closed && !a.segments.is_empty() && !b.segments.is_empty())
    }
}
