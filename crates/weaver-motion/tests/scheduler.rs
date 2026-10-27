//! Tests for the scheduler and the engine façade.

use weaver_motion::{
    AnimationEvent, MotionEngine, MotionPolicy, MotionToken, Scheduler, ValueSnapshot, Vec2,
};

fn spring(value: f64, target: f64) -> Box<dyn weaver_motion::AnimatedValue> {
    Box::new(weaver_motion::Animated::new(
        value,
        target,
        MotionToken::DefaultSpatial.spring_config().unwrap(),
    ))
}

#[test]
fn the_scheduler_stops_requesting_frames_once_everything_settled() {
    let mut scheduler = Scheduler::new();
    scheduler.insert(Some("scale".into()), spring(1.0, 1.06));

    assert!(scheduler.needs_frame());

    let mut frames = 0;
    loop {
        let snapshot = scheduler.tick(frames as f64 / 60.0);
        frames += 1;
        assert!(frames < 10_000);
        if !snapshot.needs_frame {
            break;
        }
    }

    assert!(scheduler.is_idle());
    assert!(!scheduler.needs_frame());

    // The value landed exactly on its destination.
    let value = scheduler.value_by_key("scale").unwrap();
    assert_eq!(value.as_scalar(), Some(1.06));
}

#[test]
fn registering_a_key_twice_does_not_leave_orphans() {
    let mut scheduler = Scheduler::new();
    scheduler.insert(Some("panel".into()), spring(0.0, 1.0));
    scheduler.insert(Some("panel".into()), spring(0.0, 1.0));

    assert_eq!(scheduler.len(), 1);
}

#[test]
fn removal_is_reported_as_an_event() {
    let mut scheduler = Scheduler::new();
    let id = scheduler.insert(Some("panel".into()), spring(0.0, 1.0));

    assert!(scheduler.remove(id));

    let snapshot = scheduler.tick(1.0 / 60.0);
    assert!(snapshot.events.contains(&AnimationEvent::Removed(id)));
    assert!(snapshot.entries.is_empty());
}

#[test]
fn settling_is_reported_as_an_event() {
    let mut scheduler = Scheduler::new();
    let id = scheduler.insert(Some("panel".into()), spring(0.0, 1.0));

    let mut events = Vec::new();
    for frame in 0..600 {
        let snapshot = scheduler.tick(frame as f64 / 60.0);
        events.extend(snapshot.events);
        if !snapshot.needs_frame {
            break;
        }
    }

    assert!(events.contains(&AnimationEvent::Settled(id)));
}

#[test]
fn the_engine_keeps_velocity_when_retargeting() {
    let mut engine = MotionEngine::new();
    let scale = engine.spring("scale", 1.0, MotionToken::FastSpatial);
    engine.retarget(scale, 1.06);

    // The first tick only establishes the clock's reference timestamp.
    engine.tick(0.0);
    engine.tick(1.0 / 60.0);
    let value = engine.value_f64(scale).unwrap();
    assert!(value > 1.0);

    let velocity_before = engine
        .scheduler()
        .get_animated::<f64>(scale)
        .unwrap()
        .velocity();

    engine.retarget(scale, 1.0);

    let velocity_after = engine
        .scheduler()
        .get_animated::<f64>(scale)
        .unwrap()
        .velocity();

    assert_eq!(velocity_before, velocity_after);
    assert_eq!(engine.value_f64(scale).unwrap(), value);
}

#[test]
fn the_engine_continues_an_existing_animation_when_a_key_is_reused() {
    let mut engine = MotionEngine::new();
    let first = engine.spring("scale", 1.0, MotionToken::FastSpatial);
    engine.retarget(first, 1.06);
    engine.tick(0.0);
    engine.tick(1.0 / 60.0);

    let value = engine.value_f64(first).unwrap();
    let second = engine.spring("scale", 1.0, MotionToken::FastSpatial);

    assert_ne!(first, second);
    assert_eq!(engine.value_f64(second).unwrap(), value);
    assert_eq!(engine.scheduler().len(), 1);
}

#[test]
fn vector_animations_are_supported_end_to_end() {
    let mut engine = MotionEngine::new();
    let scale = engine.spring_vec2("scale", Vec2::ONE, MotionToken::ExpressiveSpatial);

    engine.retarget_vec2(scale, Vec2::splat(1.2));
    engine.tick(0.0);
    engine.tick(1.0 / 60.0);

    let value = engine.value_vec2(scale).unwrap();
    assert!(value.x > 1.0);
    assert_eq!(value.y, value.x, "both components use the same spring");
}

#[test]
fn the_reduced_motion_policy_removes_overshoot() {
    let mut engine = MotionEngine::with_policy(MotionPolicy::Reduced);
    let scale = engine.spring("scale", 1.0, MotionToken::ExpressiveSpatial);

    engine.retarget(scale, 1.5);

    let mut maximum = 1.0_f64;
    for frame in 0..600 {
        engine.tick(frame as f64 / 60.0);
        maximum = maximum.max(engine.value_f64(scale).unwrap());
    }

    assert!(maximum <= 1.5, "reduced motion must not overshoot");
}

#[test]
fn the_off_policy_lands_immediately() {
    let mut engine = MotionEngine::with_policy(MotionPolicy::Off);
    let scale = engine.spring("scale", 1.0, MotionToken::FastSpatial);

    engine.retarget(scale, 1.5);

    assert_eq!(engine.value_f64(scale).unwrap(), 1.5);
}

#[test]
fn snapshots_expose_values_and_keys() {
    let mut engine = MotionEngine::new();
    let scale = engine.spring("scale", 1.0, MotionToken::FastSpatial);
    engine.retarget(scale, 1.06);

    let snapshot = engine.tick(1.0 / 60.0);
    let entry = snapshot.get_by_key("scale").expect("entry");
    assert!(matches!(entry.value, ValueSnapshot::Scalar(_)));
    assert!(entry.active);
    assert!(snapshot.needs_frame);
}

#[test]
fn removing_an_animation_stops_the_frame_loop() {
    let mut engine = MotionEngine::new();
    let scale = engine.spring("scale", 1.0, MotionToken::FastSpatial);
    engine.retarget(scale, 1.06);

    engine.tick(0.0);
    engine.tick(1.0 / 60.0);
    assert!(engine.needs_frame());

    assert!(engine.remove(scale));
    assert!(!engine.needs_frame());
    assert!(engine.is_idle());
}
