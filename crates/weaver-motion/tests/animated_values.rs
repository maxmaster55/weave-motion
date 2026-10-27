//! Tests for the generic animated values.

use weaver_motion::{Animated, MotionToken, SpringConfig, Vec2, Vec4};

#[test]
fn vector_springs_move_each_component() {
    let config = MotionToken::DefaultSpatial.spring_config().unwrap();
    let mut animated = Animated::new(Vec2::ZERO, Vec2::new(10.0, -4.0), config);

    animated.advance(0.1);

    assert!(animated.value().x > 0.0);
    assert!(animated.value().y < 0.0);
    assert!(animated.velocity().x > 0.0);
    assert!(animated.velocity().y < 0.0);
}

#[test]
fn vector_springs_preserve_velocity_on_retarget() {
    let config = MotionToken::ExpressiveSpatial.spring_config().unwrap();
    let mut animated = Animated::new(Vec2::ONE, Vec2::splat(2.0), config);

    animated.advance(0.05);
    let velocity = animated.velocity();
    let value = animated.value();

    animated.set_target(Vec2::ZERO);

    assert_eq!(animated.velocity(), velocity);
    assert_eq!(animated.value(), value);
}

#[test]
fn vector_springs_settle_exactly() {
    let config = MotionToken::GentleSpatial.spring_config().unwrap();
    let mut animated = Animated::new(Vec2::ZERO, Vec2::new(100.0, 50.0), config);

    let mut frames = 0;
    while animated.advance(1.0 / 60.0) {
        frames += 1;
        assert!(frames < 10_000);
    }

    assert_eq!(animated.value(), Vec2::new(100.0, 50.0));
    assert_eq!(animated.velocity(), Vec2::ZERO);
}

#[test]
fn quad_values_are_supported() {
    let config = MotionToken::FastSpatial.spring_config().unwrap();
    let mut animated = Animated::new(Vec4::ZERO, Vec4::new(1.0, 2.0, 3.0, 4.0), config);

    animated.advance(0.2);

    let value = animated.value();
    assert!(value.x > 0.0 && value.y > 0.0 && value.z > 0.0 && value.w > 0.0);
    assert!(value.x < 1.0 && value.w < 4.0);
}

#[test]
fn interpolation_helpers_behave() {
    use weaver_motion::Animatable;

    assert_eq!(Animatable::lerp(0.0_f64, 10.0_f64, 0.25), 2.5);
    assert_eq!(
        Animatable::lerp(Vec2::ZERO, Vec2::new(4.0, 8.0), 0.5),
        Vec2::new(2.0, 4.0)
    );
    assert_eq!(
        Animatable::distance(Vec2::ZERO, Vec2::new(3.0, 1.0)),
        3.0,
        "distance uses the largest component difference"
    );
}

#[test]
fn custom_spring_configurations_are_honoured() {
    let config = SpringConfig::unit_mass(1000.0, 1.0).unwrap();
    let mut animated = Animated::new(0.0_f64, 100.0, config);

    // A very stiff, critically damped spring gets close quickly and never
    // overshoots.
    for _ in 0..600 {
        animated.advance(1.0 / 120.0);
        assert!(animated.value() <= 100.0);
    }
    assert!(animated.is_settled());
}
