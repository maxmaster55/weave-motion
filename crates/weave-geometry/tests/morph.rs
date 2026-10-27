//! Tests for the geometry primitives and their use with the engine.

use weaver_motion::{Animated, MotionToken, SpringConfig};
use weave_geometry::{Extrapolation, Morph, Radii, Rect, RoundedRect, Shape};

#[test]
fn a_morph_stays_valid_over_the_whole_range() {
    let from = Shape::pill(Rect::new(0.0, 0.0, 96.0, 32.0));
    let to = Shape::rounded_rect(Rect::new(0.0, 0.0, 400.0, 280.0), Radii::uniform(28.0));
    let morph = Morph::new(from, to).expect("compatible shapes");

    for step in 0..=100 {
        let t = step as f64 / 100.0;
        let shape = morph.apply(t).expect("shape");
        let bounds = shape.bounds();

        assert!(bounds.width > 0.0 && bounds.height > 0.0);
        assert!(bounds.is_finite());

        match shape {
            Shape::RoundedRect(rounded) => {
                assert!(rounded.radii.max() <= bounds.height / 2.0 + 1e-9);
                assert!(rounded.radii.top_left + rounded.radii.top_right <= bounds.width + 1e-9);
                assert!(rounded.radii.top_left + rounded.radii.bottom_left <= bounds.height + 1e-9);
            }
        }
    }
}

#[test]
fn an_overshooting_spring_cannot_break_the_geometry() {
    let from = Shape::pill(Rect::new(0.0, 0.0, 96.0, 32.0));
    let to = Shape::rounded_rect(Rect::new(0.0, 0.0, 400.0, 280.0), Radii::uniform(28.0));
    let morph = Morph::new(from, to).unwrap();

    // An expressive spring overshoots past 1.0.
    let config = MotionToken::ExpressiveSpatial.spring_config().unwrap();
    let mut progress = Animated::new(0.0_f64, 1.0, config);
    for _ in 0..600 {
        progress.advance(1.0 / 120.0);
        let shape = morph.apply(progress.value()).expect("shape");
        assert!(shape.bounds().width >= 0.0);
        assert!(shape.bounds().height >= 0.0);
    }
}

#[test]
fn the_morph_progress_is_clamped_by_default() {
    let from = Shape::pill(Rect::new(0.0, 0.0, 96.0, 32.0));
    let to = Shape::rounded_rect(Rect::new(0.0, 0.0, 400.0, 280.0), Radii::uniform(28.0));

    let clamped = Morph::new(from, to).unwrap();
    let extrapolating = Morph::new(from, to)
        .unwrap()
        .with_extrapolation(Extrapolation::Extrapolate);

    assert_eq!(clamped.apply(1.5), clamped.apply(1.0));
    assert!(extrapolating.apply(1.5).unwrap().bounds().width > 400.0);
}

#[test]
fn a_rect_can_be_animated_by_the_engine() {
    let config = SpringConfig::unit_mass(300.0, 0.8).unwrap();
    let mut bounds = Animated::new(
        Rect::new(0.0, 0.0, 100.0, 40.0),
        Rect::new(0.0, 0.0, 320.0, 240.0),
        config,
    );

    bounds.advance(0.2);
    let value = bounds.value();
    assert!(value.width > 100.0 && value.width < 320.0);

    // Retargeting keeps the velocity.
    let velocity = bounds.velocity();
    bounds.set_target(Rect::new(0.0, 0.0, 100.0, 40.0));
    assert_eq!(bounds.velocity(), velocity);
}

#[test]
fn rounded_rects_scale_without_breaking_the_radii() {
    let rounded = RoundedRect::uniform(Rect::new(0.0, 0.0, 200.0, 100.0), 20.0);
    let scaled = rounded.scaled(0.5, 0.5);

    assert_eq!(scaled.rect.width, 100.0);
    assert_eq!(scaled.radii.top_left, 10.0);
    assert!(scaled.is_valid());
}
