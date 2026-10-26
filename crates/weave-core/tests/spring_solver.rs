//! Behavioural tests for the spring solver, written against the public API.

use weave_core::{solve, Spring, SpringConfig};

fn config() -> SpringConfig {
    SpringConfig::unit_mass(300.0, 0.8).unwrap()
}

/// Runs a spring for `duration` seconds at the given frame rate and returns
/// the final `(value, velocity)`.
fn run_at(frame_rate: f64, duration: f64, config: SpringConfig) -> (f64, f64) {
    let mut spring = Spring::new(0.0, 1.0, config);
    let dt = 1.0 / frame_rate;
    let steps = (duration * frame_rate).round() as i64;
    for _ in 0..steps {
        spring.advance(dt);
    }
    (spring.value(), spring.velocity())
}

#[test]
fn frame_rate_does_not_change_the_outcome() {
    let cfg = config();
    let reference = run_at(60.0, 1.0, cfg);

    for frame_rate in [30.0, 45.0, 90.0, 120.0, 144.0] {
        let (value, velocity) = run_at(frame_rate, 1.0, cfg);
        assert!(
            (value - reference.0).abs() < 1e-6,
            "{frame_rate} Hz: value {value} != {}",
            reference.0
        );
        assert!(
            (velocity - reference.1).abs() < 1e-6,
            "{frame_rate} Hz: velocity {velocity} != {}",
            reference.1
        );
    }
}

#[test]
fn random_frame_jitter_reproduces_the_same_state() {
    let cfg = config();
    let mut jittered = Spring::new(0.0, 1.0, cfg);

    // A deterministic pseudo random sequence of frame gaps.
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    let mut next_gap = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        0.004 + (seed % 40) as f64 * 0.001
    };

    let mut elapsed = 0.0;
    while elapsed < 1.0 {
        let dt = next_gap().min(1.0 - elapsed);
        jittered.advance(dt);
        elapsed += dt;
    }

    let (reference, _) = run_at(60.0, 1.0, cfg);
    assert!((jittered.value() - reference).abs() < 1e-3);
}

#[test]
fn interrupting_a_motion_keeps_position_and_velocity() {
    let mut spring = Spring::new(0.0, 1.0, config());

    spring.advance(0.05);
    let (value, velocity) = (spring.value(), spring.velocity());

    // The destination changes while the value is in flight.
    spring.set_target(0.4);

    assert_eq!(spring.value(), value);
    assert_eq!(spring.velocity(), velocity);

    // The motion continues from the interrupted state.
    spring.advance(1.0 / 60.0);
    assert!(spring.value() != value);
    assert!(spring.velocity().abs() > 0.0);
}

#[test]
fn reversing_direction_never_reverses_instantaneously() {
    let mut spring = Spring::new(0.0, 1.0, config());
    spring.advance(0.1);
    assert!(spring.velocity() > 0.0);

    spring.set_target(0.0);
    spring.advance(1e-6);

    // Velocity is still positive right after the retarget: the spring has to
    // decelerate first. This is what makes an interrupted animation look
    // physical instead of snapping.
    assert!(spring.velocity() > 0.0);
}

#[test]
fn a_long_suspension_is_caught_up_in_one_step() {
    let cfg = config();
    let mut spring = Spring::new(0.0, 1.0, cfg);

    spring.advance(5.0);

    assert!(spring.is_settled());
    assert_eq!(spring.value(), 1.0);
}

#[test]
fn settling_snapshots_are_exact() {
    let mut spring = Spring::new(0.0, 1.0, config());
    while spring.advance(1.0 / 60.0) {}

    assert_eq!(spring.value(), 1.0);
    assert_eq!(spring.velocity(), 0.0);

    // A settled spring stays settled and reports no motion.
    assert!(!spring.advance(1.0 / 60.0));
    assert_eq!(spring.value(), 1.0);
}

#[test]
fn solve_is_pure() {
    let cfg = config();
    let a = solve(&cfg, 0.25, 1.5, 1.0, 0.1);
    let b = solve(&cfg, 0.25, 1.5, 1.0, 0.1);
    assert_eq!(a, b);
}
