//! Tests for the scene layer.

use weave_geometry::{Rect, Shape};
use weave_scene::{Cue, Direction, Gate, Scene, TransitionMode};
use weaver_motion::MotionPolicy;

fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect::new(x, y, width, height)
}

#[test]
fn element_identity_survives_a_rebuild() {
    let mut scene = Scene::new();
    let first = scene.layout("button", rect(0.0, 0.0, 100.0, 40.0));
    let other = scene.layout("label", rect(0.0, 50.0, 100.0, 20.0));

    assert_ne!(first, other);

    // The UI tree is rebuilt: the same key yields the same identity.
    let again = scene.layout("button", rect(0.0, 0.0, 100.0, 40.0));
    assert_eq!(first, again);
    assert_eq!(scene.len(), 2);
}

#[test]
fn the_first_placement_does_not_animate() {
    let mut scene = Scene::new();
    scene.layout("button", rect(10.0, 20.0, 100.0, 40.0));

    let presentation = scene.presentation("button").expect("presentation");
    assert_eq!(presentation.bounds, rect(10.0, 20.0, 100.0, 40.0));
    assert!(scene.is_idle(), "nothing should be moving yet");
}

#[test]
fn a_layout_change_animates_the_bounds() {
    let mut scene = Scene::new();
    scene.layout("button", rect(0.0, 0.0, 100.0, 40.0));
    scene.layout("button", rect(0.0, 0.0, 300.0, 200.0));

    let start = scene.presentation("button").unwrap().bounds.width;
    scene.tick(0.0);
    scene.tick(1.0 / 60.0);

    let current = scene.presentation("button").unwrap().bounds.width;
    assert!(
        current > start,
        "the bounds should move towards the destination"
    );
    assert!(current < 300.0, "the bounds should not jump");
}

#[test]
fn interrupting_a_transition_keeps_the_velocity() {
    let mut scene = Scene::new();
    scene.layout("panel", rect(0.0, 0.0, 100.0, 40.0));
    scene.layout("panel", rect(0.0, 0.0, 400.0, 300.0));

    scene.tick(0.0);
    scene.tick(1.0 / 60.0);
    scene.tick(2.0 / 60.0);

    let element = scene.element_id("panel").unwrap();
    let velocity_before = scene
        .engine()
        .scheduler()
        .get_animated::<Rect>(scene.engine().scheduler().ids().next().unwrap())
        .map(|animated| animated.velocity())
        .unwrap();

    // The destination changes again while the panel is in flight.
    scene.layout("panel", rect(0.0, 0.0, 200.0, 150.0));

    let velocity_after = scene
        .engine()
        .scheduler()
        .get_animated::<Rect>(scene.engine().scheduler().ids().next().unwrap())
        .map(|animated| animated.velocity())
        .unwrap();

    assert_eq!(velocity_before, velocity_after);
    assert!(element.raw() > 0);
}

#[test]
fn flip_mode_moves_the_element_with_a_transform() {
    let mut scene = Scene::new();
    scene.layout_with_mode("card", rect(0.0, 0.0, 100.0, 50.0), TransitionMode::Flip);
    scene.tick(0.0);

    scene.layout_with_mode(
        "card",
        rect(400.0, 200.0, 300.0, 150.0),
        TransitionMode::Flip,
    );

    let presentation = scene.presentation("card").unwrap();
    assert_eq!(presentation.bounds, rect(400.0, 200.0, 300.0, 150.0));
    assert!(
        presentation.transform.translate_x > 0.0,
        "the flip starts translated back to the old position"
    );

    scene.tick(1.0 / 60.0);
    scene.tick(2.0 / 60.0);

    let presentation = scene.presentation("card").unwrap();
    assert!(
        presentation.transform.translate_x < 500.0,
        "the transform animates towards the identity"
    );
}

#[test]
fn a_shape_morph_runs_from_the_current_shape() {
    let mut scene = Scene::new();
    scene.layout("surface", rect(0.0, 0.0, 120.0, 40.0));
    scene.shape("surface", Shape::pill(rect(0.0, 0.0, 120.0, 40.0)));

    scene.tick(0.0);
    scene.tick(1.0 / 60.0);

    let presentation = scene.presentation("surface").unwrap();
    assert!(presentation.morph_progress > 0.0);
    assert!(presentation.clip.is_some());
}

#[test]
fn choreography_fires_when_the_gate_opens() {
    let mut scene = Scene::new();
    scene.layout("panel", rect(0.0, 0.0, 100.0, 40.0));
    scene.layout("panel", rect(0.0, 0.0, 400.0, 300.0));

    let panel = scene.element_id("panel").unwrap();
    scene.layout("title", rect(0.0, 0.0, 0.0, 0.0));
    scene.opacity("title", 0.0);
    scene.set_content_visible("title", false);

    scene.cue(
        Gate::Progress {
            element: panel,
            threshold: 0.5,
            direction: Direction::Rising,
        },
        Cue::RevealContent {
            element: scene.element_id("title").unwrap(),
        },
    );

    // The gate is closed at the beginning.
    scene.tick(0.0);
    scene.tick(1.0 / 60.0);
    assert!(
        !scene.presentation("title").unwrap().content_visible,
        "the content must not appear before the panel is wide enough"
    );

    // Let the transition run: the gate opens and the content appears.
    for frame in 2..120 {
        scene.tick(frame as f64 / 60.0);
        if scene.presentation("title").unwrap().content_visible {
            break;
        }
    }

    assert!(
        scene.presentation("title").unwrap().content_visible,
        "the content should be revealed once the panel is wide enough"
    );
}

#[test]
fn repeated_layout_changes_do_not_leak_animations() {
    let mut scene = Scene::new();
    scene.layout("panel", rect(0.0, 0.0, 100.0, 40.0));

    for step in 0..50 {
        scene.layout("panel", rect(0.0, 0.0, 100.0 + step as f64, 40.0));
        scene.shape(
            "panel",
            Shape::pill(rect(0.0, 0.0, 100.0 + step as f64, 40.0)),
        );
    }

    // One element: bounds, transform, opacity and one morph.
    assert_eq!(scene.engine().scheduler().len(), 4);
}

#[test]
fn the_scene_settles_and_goes_idle() {
    let mut scene = Scene::new();
    scene.layout("panel", rect(0.0, 0.0, 100.0, 40.0));
    scene.layout("panel", rect(0.0, 0.0, 400.0, 300.0));

    let mut frames = 0;
    loop {
        scene.tick(frames as f64 / 60.0);
        frames += 1;
        assert!(frames < 2_000, "the scene never settled");
        if scene.is_idle() {
            break;
        }
    }

    let presentation = scene.presentation("panel").unwrap();
    assert_eq!(presentation.bounds.width, 400.0);
    assert_eq!(presentation.bounds.height, 300.0);
}

#[test]
fn reduced_motion_shortens_the_transition() {
    let mut scene = Scene::with_policy(MotionPolicy::Reduced);
    scene.layout("panel", rect(0.0, 0.0, 100.0, 40.0));
    scene.layout("panel", rect(0.0, 0.0, 400.0, 300.0));

    scene.tick(0.0);
    scene.tick(1.0 / 60.0);

    let presentation = scene.presentation("panel").unwrap();
    assert!(presentation.bounds.width > 100.0);
    assert!(
        presentation.bounds.width <= 400.0,
        "reduced motion must not overshoot the destination"
    );
}
