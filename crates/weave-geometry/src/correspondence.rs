//! Correspondence between two contours.
//!
//! Two outlines can only be interpolated point by point when their points
//! actually correspond. Weave establishes that correspondence in four
//! steps:
//!
//! 1. both contours are flattened and resampled by **arc length**, so the
//!    points are equally spaced instead of equally indexed;
//! 2. the **winding** is normalised, so both contours run in the same
//!    direction;
//! 3. the **start points** are aligned by searching the cyclic offset that
//!    minimises the total distance;
//! 4. the result is validated: a morph that twists or self-intersects is
//!    refused instead of producing garbage.

use alloc::vec::Vec;

use crate::path::Contour;
use crate::point::Point;

const EPSILON: f64 = 1e-12;
const SAMPLES_PER_SEGMENT: usize = 16;

/// Flattens a contour into a polyline.
fn flatten(contour: &Contour) -> Vec<Point> {
    let mut points = Vec::new();
    for segment in &contour.segments {
        for step in 0..SAMPLES_PER_SEGMENT {
            points.push(segment.point(step as f64 / SAMPLES_PER_SEGMENT as f64));
        }
    }
    if contour.closed {
        if let Some(last) = contour.segments.last() {
            points.push(last.p3);
        }
    }
    points
}

/// Resamples a contour into `samples` points that are equally spaced by
/// arc length.
///
/// Points that are equally spaced by *index* carry no meaning: a long edge
/// and a short edge would contribute the same number of points.
pub fn resample(contour: &Contour, samples: usize) -> Vec<Point> {
    let samples = samples.max(3);
    let polyline = flatten(contour);

    if polyline.len() < 2 {
        return polyline;
    }

    let mut cumulative = Vec::with_capacity(polyline.len());
    let mut total = 0.0;
    cumulative.push(0.0);
    for index in 1..polyline.len() {
        total += polyline[index].distance(polyline[index - 1]);
        cumulative.push(total);
    }

    if total <= EPSILON {
        return alloc::vec![polyline[0]; samples];
    }

    let step = total / samples as f64;
    let mut result = Vec::with_capacity(samples);
    let mut index = 0;

    for sample in 0..samples {
        let target = sample as f64 * step;
        while index + 1 < cumulative.len() && cumulative[index + 1] < target {
            index += 1;
        }
        let next = (index + 1).min(cumulative.len() - 1);
        let segment_length = cumulative[next] - cumulative[index];
        let t = if segment_length > EPSILON {
            ((target - cumulative[index]) / segment_length).clamp(0.0, 1.0)
        } else {
            0.0
        };
        result.push(polyline[index].lerp(polyline[next], t));
    }

    result
}

/// The signed area of a polygon given by its points.
pub fn signed_area(points: &[Point]) -> f64 {
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

/// Makes sure the points run in the same winding direction.
///
/// The direction of the first point set is kept; the second one is flipped
/// if necessary.
pub fn normalize_winding(reference: &[Point], points: &mut [Point]) {
    let reference_area = signed_area(reference);
    let area = signed_area(points);
    if reference_area * area < 0.0 {
        points.reverse();
    }
}

/// Finds the cyclic offset that aligns `to` with `from` best.
///
/// The offset with the smallest total squared distance wins. Because the
/// search is cyclic, the contours may start at completely different
/// places.
pub fn best_shift(from: &[Point], to: &[Point]) -> usize {
    let count = from.len().min(to.len());
    if count == 0 {
        return 0;
    }

    let mut best_shift = 0;
    let mut best_cost = f64::MAX;

    for shift in 0..count {
        let mut cost = 0.0;
        for index in 0..count {
            cost += from[index].distance_squared(to[(index + shift) % count]);
        }
        if cost < best_cost {
            best_cost = cost;
            best_shift = shift;
        }
    }

    best_shift
}

/// Rotates the points so that they start at `offset`.
pub fn rotate(points: &[Point], offset: usize) -> Vec<Point> {
    if points.is_empty() {
        return Vec::new();
    }
    let count = points.len();
    (0..count)
        .map(|index| points[(index + offset) % count])
        .collect()
}

/// The average squared distance between two point sets.
pub fn alignment_cost(from: &[Point], to: &[Point]) -> f64 {
    let count = from.len().min(to.len());
    if count == 0 {
        return f64::MAX;
    }
    let mut cost = 0.0;
    for index in 0..count {
        cost += from[index].distance_squared(to[index]);
    }
    cost / count as f64
}

/// Whether two segments cross each other.
fn segments_cross(a0: Point, a1: Point, b0: Point, b1: Point) -> bool {
    let cross =
        |o: Point, p: Point, q: Point| (q.x - o.x) * (p.y - o.y) - (q.y - o.y) * (p.x - o.x);

    let d1 = cross(a0, a1, b0);
    let d2 = cross(a0, a1, b1);
    let d3 = cross(b0, b1, a0);
    let d4 = cross(b0, b1, a1);

    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }

    // Touching counts as an intersection as well: a shape that touches
    // itself is not a valid intermediate frame.
    let on_segment = |p: Point, q: Point, r: Point| {
        p.x <= q.x.max(r.x) + EPSILON
            && p.x >= q.x.min(r.x) - EPSILON
            && p.y <= q.y.max(r.y) + EPSILON
            && p.y >= q.y.min(r.y) - EPSILON
    };

    (d1.abs() <= EPSILON && on_segment(b0, a0, a1))
        || (d2.abs() <= EPSILON && on_segment(b1, a0, a1))
        || (d3.abs() <= EPSILON && on_segment(a0, b0, b1))
        || (d4.abs() <= EPSILON && on_segment(a1, b0, b1))
}

/// Whether a closed polygon intersects itself.
pub fn has_self_intersection(points: &[Point]) -> bool {
    let count = points.len();
    if count < 4 {
        return false;
    }

    for first in 0..count {
        let a0 = points[first];
        let a1 = points[(first + 1) % count];
        for second in (first + 1)..count {
            // Neighbouring edges share an endpoint, that is not a crossing.
            if second == first || (second + 1) % count == first || second == (first + 1) % count {
                continue;
            }
            let b0 = points[second];
            let b1 = points[(second + 1) % count];
            if segments_cross(a0, a1, b0, b1) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::Path;

    fn circle(samples: usize, radius: f64) -> Vec<Point> {
        (0..samples)
            .map(|index| {
                let angle = index as f64 / samples as f64 * core::f64::consts::TAU;
                Point::new(angle.cos() * radius, angle.sin() * radius)
            })
            .collect()
    }

    #[test]
    fn resampling_produces_equally_spaced_points() {
        let path = Path::from_points(&circle(64, 10.0), true);
        let contour = &path.contours[0];
        let points = resample(contour, 32);

        assert_eq!(points.len(), 32);

        let mut distances = Vec::new();
        for index in 0..points.len() {
            distances.push(points[index].distance(points[(index + 1) % points.len()]));
        }
        let average = distances.iter().sum::<f64>() / distances.len() as f64;
        for distance in distances {
            assert!(
                (distance - average).abs() < average * 0.1,
                "the samples are not equally spaced"
            );
        }
    }

    #[test]
    fn the_winding_is_normalised() {
        let mut clockwise = circle(16, 1.0);
        clockwise.reverse();

        let reference = circle(16, 1.0);
        normalize_winding(&reference, &mut clockwise);

        assert!(signed_area(&reference) * signed_area(&clockwise) > 0.0);
    }

    #[test]
    fn the_start_points_are_aligned() {
        let reference = circle(32, 10.0);

        // The same circle, but starting at a different point.
        let mut shifted = circle(32, 10.0);
        shifted.rotate_left(11);

        normalize_winding(&reference, &mut shifted);
        let shift = best_shift(&reference, &shifted);
        let aligned = rotate(&shifted, shift);

        assert!(alignment_cost(&reference, &aligned) < 1e-6);
    }

    #[test]
    fn a_self_intersecting_polygon_is_detected() {
        let bowtie = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(10.0, 0.0),
            Point::new(0.0, 10.0),
        ];
        assert!(has_self_intersection(&bowtie));

        let square = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ];
        assert!(!has_self_intersection(&square));
    }
}
