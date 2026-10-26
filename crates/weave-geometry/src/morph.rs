//! Morphing between compatible contours.
//!
//! A [`MorphPlan`] is the *correspondence* between two paths: the work of
//! matching their points is done once, when the plan is built, and the
//! plan can then be evaluated for every frame of the transition.
//!
//! The plan is refused when the two paths cannot be matched safely. In that
//! case the caller has to fall back to a crossfade — a broken outline is
//! worse than a fade.

use alloc::vec::Vec;

use crate::correspondence::{
    alignment_cost, best_shift, has_self_intersection, normalize_winding, resample, rotate,
    signed_area,
};
use crate::path::{Contour, Path};
use crate::point::{CubicSegment, Point};
use crate::rect::Rect;

/// The number of points used per contour when no other value is given.
pub const DEFAULT_SAMPLES: usize = 64;

/// A morph between two compatible paths.
///
/// The plan works in a normalized unit box: both paths are mapped into
/// `[0, 1]²` before the correspondence is computed, so the plan is
/// independent of the size of the shapes and can be reused for any output
/// bounds.
#[derive(Clone, Debug, PartialEq)]
pub struct MorphPlan {
    from: Vec<CubicSegment>,
    to: Vec<CubicSegment>,
    source_bounds: Rect,
    target_bounds: Rect,
    cost: f64,
}

impl MorphPlan {
    /// Builds the correspondence between two paths.
    ///
    /// Returns `None` when the paths are not compatible or when the
    /// resulting morph would be unsafe (a twist or a self-intersection).
    pub fn build(from: &Path, to: &Path) -> Option<Self> {
        Self::build_with_samples(from, to, DEFAULT_SAMPLES)
    }

    /// Builds the correspondence with a specific number of samples.
    pub fn build_with_samples(from: &Path, to: &Path, samples: usize) -> Option<Self> {
        if !from.is_compatible_with(to) {
            return None;
        }

        let source_bounds = from.bounds();
        let target_bounds = to.bounds();

        let mut from_segments = Vec::new();
        let mut to_segments = Vec::new();
        let mut total_cost = 0.0_f64;
        let mut contour_count = 0usize;

        for (from_contour, to_contour) in from.contours.iter().zip(to.contours.iter()) {
            // The cyclic correspondence assumes closed contours.
            if !from_contour.closed || !to_contour.closed {
                return None;
            }

            let source_points = resample(from_contour, samples);
            let mut target_points = resample(to_contour, samples);

            if source_points.len() < 3 || target_points.len() < 3 {
                return None;
            }

            // Normalize the winding so both contours run in the same
            // direction, then align their start points.
            normalize_winding(&source_points, &mut target_points);
            let shift = best_shift(&source_points, &target_points);
            let target_points = rotate(&target_points, shift);

            let cost = alignment_cost(&source_points, &target_points);
            total_cost += cost;
            contour_count += 1;

            let source_points = map_to_unit_box(&source_points, &source_bounds);
            let target_points = map_to_unit_box(&target_points, &target_bounds);

            from_segments.extend(points_to_cubics(&source_points, from_contour.closed));
            to_segments.extend(points_to_cubics(&target_points, to_contour.closed));
        }

        let plan = Self {
            from: from_segments,
            to: to_segments,
            source_bounds,
            target_bounds,
            cost: if contour_count > 0 {
                total_cost / contour_count as f64
            } else {
                0.0
            },
        };

        if !plan.is_safe() {
            return None;
        }

        Some(plan)
    }

    /// The bounding box of the source path.
    pub fn source_bounds(&self) -> Rect {
        self.source_bounds
    }

    /// The bounding box of the destination path.
    pub fn target_bounds(&self) -> Rect {
        self.target_bounds
    }

    /// The average squared distance between the matched points.
    ///
    /// A high value means the shapes are far apart: the morph will look
    /// like a transformation rather than a subtle change.
    pub fn alignment_cost(&self) -> f64 {
        self.cost
    }

    /// Whether the morph stays valid over the whole range.
    ///
    /// The plan is built so that this is always true; the check exists so
    /// that a future change cannot silently break it.
    pub fn is_safe(&self) -> bool {
        for step in 0..=8 {
            let progress = step as f64 / 8.0;
            let points = self.points(progress);
            if has_self_intersection(&points) {
                return false;
            }
            if signed_area(&points).abs() < 1e-9 {
                return false;
            }
        }
        true
    }

    /// The interpolated points of the morph, in the unit box.
    pub fn points(&self, progress: f64) -> Vec<Point> {
        let progress = progress.clamp(0.0, 1.0);
        let mut points = Vec::with_capacity(self.from.len());

        for (from, to) in self.from.iter().zip(self.to.iter()) {
            points.push(from.p0.lerp(to.p0, progress));
        }

        points
    }

    /// The interpolated path, in the unit box.
    pub fn path(&self, progress: f64) -> Path {
        self.path_in(progress, Rect::new(0.0, 0.0, 1.0, 1.0))
    }

    /// The interpolated path, mapped into `bounds`.
    pub fn path_in(&self, progress: f64, bounds: Rect) -> Path {
        let progress = progress.clamp(0.0, 1.0);

        let mut contours = Vec::new();
        let mut current = Contour::default();

        for (from, to) in self.from.iter().zip(self.to.iter()) {
            let p0 = from.p0.lerp(to.p0, progress);
            let p1 = from.p1.lerp(to.p1, progress);
            let p2 = from.p2.lerp(to.p2, progress);
            let p3 = from.p3.lerp(to.p3, progress);

            current.segments.push(CubicSegment::new(
                map_point(p0, bounds),
                map_point(p1, bounds),
                map_point(p2, bounds),
                map_point(p3, bounds),
            ));
            current.closed = true;
        }

        contours.push(current);
        Path::new(contours)
    }
}

/// Maps a point from the unit box into `bounds`.
fn map_point(point: Point, bounds: Rect) -> Point {
    Point::new(
        bounds.x + point.x * bounds.width,
        bounds.y + point.y * bounds.height,
    )
}

/// Maps points into the unit box, using `bounds` as the reference frame.
fn map_to_unit_box(points: &[Point], bounds: &Rect) -> Vec<Point> {
    if bounds.width.abs() <= f64::EPSILON || bounds.height.abs() <= f64::EPSILON {
        return points.to_vec();
    }
    points
        .iter()
        .map(|point| {
            Point::new(
                (point.x - bounds.x) / bounds.width,
                (point.y - bounds.y) / bounds.height,
            )
        })
        .collect()
}

/// Converts a list of points into cubic segments.
///
/// The control points sit on the segment itself, so the polyline is
/// preserved exactly while every edge becomes a cubic that can be
/// interpolated point by point.
fn points_to_cubics(points: &[Point], closed: bool) -> Vec<CubicSegment> {
    let mut segments = Vec::new();
    if points.len() < 2 {
        return segments;
    }

    let count = if closed {
        points.len()
    } else {
        points.len() - 1
    };
    for index in 0..count {
        let from = points[index];
        let to = points[(index + 1) % points.len()];
        let dx = (to.x - from.x) / 3.0;
        let dy = (to.y - from.y) / 3.0;

        segments.push(CubicSegment::new(
            from,
            Point::new(from.x + dx, from.y + dy),
            Point::new(from.x + 2.0 * dx, from.y + 2.0 * dy),
            to,
        ));
    }

    segments
}
