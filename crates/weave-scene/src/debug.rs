//! Debugging helpers.
//!
//! The engine is deliberately quiet: it does not log, it does not allocate
//! strings per frame. When something looks wrong, the scene can describe
//! its own state on demand.

use alloc::string::String;
use alloc::vec::Vec;

use weave_geometry::Shape;

use crate::scene::{Scene, SceneSnapshot};

/// The state of one element, as text.
#[derive(Clone, Debug, PartialEq)]
pub struct DebugEntry {
    /// The key of the element.
    pub key: String,
    /// The animated bounds.
    pub bounds: String,
    /// The velocity of the bounds animation.
    pub velocity: String,
    /// The progress of the shape morph.
    pub morph_progress: f64,
    /// The animated opacity.
    pub opacity: f64,
    /// Whether the content is visible.
    pub content_visible: bool,
}

impl DebugEntry {
    /// A one line description of the element.
    pub fn to_line(&self) -> String {
        let mut line = String::new();
        line.push_str(&self.key);
        line.push_str("  bounds=");
        line.push_str(&self.bounds);
        line.push_str("  velocity=");
        line.push_str(&self.velocity);
        line.push_str(&format!(
            "  morph={:.3}  opacity={:.3}  content={}",
            self.morph_progress, self.opacity, self.content_visible
        ));
        line
    }
}

/// Describes every element of a snapshot.
pub fn describe(snapshot: &SceneSnapshot) -> Vec<DebugEntry> {
    snapshot
        .presentations
        .iter()
        .map(|presentation| DebugEntry {
            key: presentation.key.clone(),
            bounds: format!(
                "{:.1}x{:.1}@{:.1},{:.1}",
                presentation.bounds.width,
                presentation.bounds.height,
                presentation.bounds.x,
                presentation.bounds.y
            ),
            velocity: String::from("-"),
            morph_progress: presentation.morph_progress,
            opacity: presentation.opacity,
            content_visible: presentation.content_visible,
        })
        .collect()
}

/// The radius of the clip shape of a presentation, if it has one.
pub fn clip_radius(presentation: &crate::Presentation) -> Option<f64> {
    match presentation.clip {
        Some(Shape::RoundedRect(rounded)) => Some(rounded.radii.top_left),
        _ => None,
    }
}

/// A human readable report of the scene.
///
/// ```
/// use weave_geometry::Rect;
/// use weave_scene::Scene;
///
/// let mut scene = Scene::new();
/// scene.layout("button", Rect::new(0.0, 0.0, 120.0, 40.0));
///
/// let report = scene.debug_report();
/// assert!(report.contains("button"));
/// ```
pub fn report(scene: &Scene, snapshot: &SceneSnapshot) -> String {
    let mut out = String::new();

    for presentation in &snapshot.presentations {
        out.push_str(&presentation.key);
        out.push_str("  bounds=");
        out.push_str(&format!(
            "{:.1}x{:.1}@{:.1},{:.1}",
            presentation.bounds.width,
            presentation.bounds.height,
            presentation.bounds.x,
            presentation.bounds.y
        ));

        if let Some(velocity) = scene.bounds_velocity(&presentation.key) {
            out.push_str(&format!("  velocity={:.1},{:.1}", velocity.x, velocity.y));
        }

        if let Some(radius) = clip_radius(presentation) {
            out.push_str(&format!("  radius={radius:.1}"));
        }

        out.push_str(&format!(
            "  morph={:.3}  opacity={:.3}  content={}",
            presentation.morph_progress, presentation.opacity, presentation.content_visible
        ));
        out.push('\n');
    }

    out.push_str(&format!(
        "idle: {}  needs_frame: {}",
        scene.is_idle(),
        scene.needs_frame()
    ));

    out
}
