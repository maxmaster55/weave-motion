//! Tests for the contour based morphing.

use weave_geometry::{
    has_self_intersection, signed_area, MorphCache, MorphCacheKey, MorphPlan, Path, Point,
};

fn circle(radius: f64, samples: usize) -> Path {
    let points: Vec<Point> = (0..samples)
        .map(|index| {
            let angle = index as f64 / samples as f64 * core::f64::consts::TAU;
            Point::new(angle.cos() * radius, angle.sin() * radius)
        })
        .collect();
    Path::from_points(&points, true)
}

fn square(size: f64) -> Path {
    Path::from_points(
        &[
            Point::new(0.0, 0.0),
            Point::new(size, 0.0),
            Point::new(size, size),
            Point::new(0.0, size),
        ],
        true,
    )
}

fn hexagon(radius: f64) -> Path {
    let points: Vec<Point> = (0..6)
        .map(|index| {
            let angle = index as f64 / 6.0 * core::f64::consts::TAU;
            Point::new(angle.cos() * radius, angle.sin() * radius)
        })
        .collect();
    Path::from_points(&points, true)
}

#[test]
fn a_circle_morphs_into_a_square() {
    let plan = MorphPlan::build(&circle(50.0, 64), &square(100.0)).expect("a valid plan");

    for step in 0..=20 {
        let progress = step as f64 / 20.0;
        let path = plan.path(progress);
        let points = plan.points(progress);

        assert!(!path.contours.is_empty());
        assert!(path.contours[0].closed, "closed loops stay closed");

        let bounds = path.bounds();
        assert!(bounds.is_finite(), "the bounds must stay finite");
        assert!(
            !has_self_intersection(&points),
            "the shape must not cross itself"
        );
        assert!(
            signed_area(&points).abs() > 1e-6,
            "the shape must not collapse"
        );
    }
}

#[test]
fn the_morph_reaches_both_endpoints() {
    let from = circle(50.0, 64);
    let to = square(100.0);
    let plan = MorphPlan::build(&from, &to).expect("a valid plan");

    // The plan works in the unit box, so both endpoints have the same
    // bounds: what changes is the outline itself.
    let start = plan.path(0.0).contours[0].points();
    let end = plan.path(1.0).contours[0].points();

    let difference: f64 = start
        .iter()
        .zip(end.iter())
        .map(|(a, b)| a.distance_squared(*b))
        .sum();
    assert!(difference > 0.1, "the two endpoints must differ");
}

#[test]
fn the_start_points_are_aligned_before_the_interpolation() {
    // The same hexagon, but with a different starting point.
    let mut shifted: Vec<Point> = (0..6)
        .map(|index| {
            let angle = index as f64 / 6.0 * core::f64::consts::TAU;
            Point::new(angle.cos() * 50.0, angle.sin() * 50.0)
        })
        .collect();
    shifted.rotate_left(3);

    let plan =
        MorphPlan::build(&hexagon(50.0), &Path::from_points(&shifted, true)).expect("a valid plan");

    // The alignment is good: the shapes almost coincide, so the average
    // distance between the matched points is small.
    assert!(
        plan.alignment_cost() < 1.0,
        "cost was {}",
        plan.alignment_cost()
    );
}

#[test]
fn incompatible_paths_are_refused() {
    // A path with two contours cannot be morphed into a path with one.
    let mut two_contours = square(10.0).clone();
    two_contours.contours.push(square(10.0).contours[0].clone());

    assert!(MorphPlan::build(&two_contours, &square(10.0)).is_none());
}

#[test]
fn open_contours_are_refused() {
    let open = Path::from_points(
        &[
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
        ],
        false,
    );

    assert!(MorphPlan::build(&open, &open).is_none());
}

#[test]
fn the_progress_is_clamped() {
    let plan = MorphPlan::build(&circle(50.0, 64), &square(100.0)).expect("a valid plan");

    let beyond = plan.path(1.5);
    let end = plan.path(1.0);
    assert_eq!(beyond, end);
}

#[test]
fn the_cache_reuses_the_plan() {
    let mut cache = MorphCache::new();
    let key = MorphCacheKey::new(1, 2, 64);

    let from = circle(50.0, 64);
    let to = square(100.0);

    assert!(cache.get_or_build(key, &from, &to).is_some());
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.misses(), 1);

    assert!(cache.get_or_build(key, &from, &to).is_some());
    assert_eq!(cache.hits(), 1);
    assert_eq!(cache.misses(), 1);
    assert_eq!(cache.len(), 1);
}

#[test]
fn a_morph_can_be_mapped_into_any_bounds() {
    let plan = MorphPlan::build(&circle(50.0, 64), &square(100.0)).expect("a valid plan");

    let bounds = weave_geometry::Rect::new(10.0, 20.0, 300.0, 200.0);
    let path = plan.path_in(0.5, bounds);
    let mapped = path.bounds();

    assert!(mapped.x >= bounds.x - 1e-6);
    assert!(mapped.y >= bounds.y - 1e-6);
    assert!(mapped.max_x() <= bounds.max_x() + 1e-6);
    assert!(mapped.max_y() <= bounds.max_y() + 1e-6);
}
