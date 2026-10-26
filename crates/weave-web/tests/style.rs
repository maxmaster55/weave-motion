//! Tests for the CSS computation of the web adapter.

use weave_geometry::{Radii, Rect};
use weave_scene::{Scene, TransitionMode};
use weave_web::DomStyle;

#[test]
fn a_presentation_becomes_absolute_positioning() {
    let mut scene = Scene::new();
    scene.layout("card", Rect::new(10.0, 20.0, 100.0, 40.0));
    let snapshot = scene.tick(0.0);

    let style = DomStyle::for_presentation(snapshot.get_by_key("card").unwrap());

    assert_eq!(style.left, "10px");
    assert_eq!(style.top, "20px");
    assert_eq!(style.width, "100px");
    assert_eq!(style.height, "40px");
    assert_eq!(style.opacity, "1");
    assert!(style.clip_path.is_none());
    assert!(style.transform.is_none());
}

#[test]
fn a_rounded_shape_becomes_a_clip_path() {
    let mut scene = Scene::new();
    scene.layout("card", Rect::new(0.0, 0.0, 100.0, 40.0));
    scene.radii("card", Radii::uniform(8.0));
    let snapshot = scene.tick(0.0);

    let style = DomStyle::for_presentation(snapshot.get_by_key("card").unwrap());

    assert_eq!(
        style.clip_path.as_deref(),
        Some("inset(0px 0px 0px 0px round 8px 8px 8px 8px)")
    );
}

#[test]
fn a_flip_transition_becomes_a_transform() {
    let mut scene = Scene::new();
    scene.layout_with_mode(
        "card",
        Rect::new(0.0, 0.0, 100.0, 50.0),
        TransitionMode::Flip,
    );
    scene.tick(0.0);
    scene.layout_with_mode(
        "card",
        Rect::new(400.0, 200.0, 100.0, 50.0),
        TransitionMode::Flip,
    );
    let snapshot = scene.tick(1.0 / 60.0);

    let style = DomStyle::for_presentation(snapshot.get_by_key("card").unwrap());

    let transform = style.transform.expect("a transform");
    assert!(transform.contains("translate("), "{transform}");
    assert!(transform.contains("scale("), "{transform}");
}

#[test]
fn the_opacity_is_clamped() {
    let mut scene = Scene::new();
    scene.layout("card", Rect::new(0.0, 0.0, 10.0, 10.0));
    let snapshot = scene.tick(0.0);

    let mut presentation = snapshot.get_by_key("card").unwrap().clone();
    presentation.opacity = 1.7;

    let style = DomStyle::for_presentation(&presentation);
    assert_eq!(style.opacity, "1");
}

#[test]
fn the_css_text_contains_every_declaration() {
    let mut scene = Scene::new();
    scene.layout("card", Rect::new(1.0, 2.0, 3.0, 4.0));
    scene.radii("card", Radii::uniform(1.0));
    let snapshot = scene.tick(0.0);

    let style = DomStyle::for_presentation(snapshot.get_by_key("card").unwrap());
    let css = style.to_css();

    assert!(css.contains("left: 1px"));
    assert!(css.contains("top: 2px"));
    assert!(css.contains("width: 3px"));
    assert!(css.contains("height: 4px"));
    assert!(css.contains("opacity: 1"));
    assert!(css.contains("clip-path: inset("));
}
