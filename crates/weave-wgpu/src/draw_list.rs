//! The renderer independent draw list.
//!
//! Turning a [`SceneSnapshot`] into a list of draw items is pure data
//! manipulation: it needs no GPU, no window and no device. That makes it
//! testable on a build machine, and it lets an application inspect what
//! *would* be drawn before handing it to a renderer.

use weave_geometry::{Radii, Rect, RoundedRect, Shape};
use weave_scene::{ElementId, SceneSnapshot};

/// A colour with straight (non premultiplied) alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    /// Red channel.
    pub r: f32,
    /// Green channel.
    pub g: f32,
    /// Blue channel.
    pub b: f32,
    /// Alpha channel.
    pub a: f32,
}

impl Color {
    /// Opaque white.
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    /// Transparent black.
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Creates a colour.
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// The colour with the alpha replaced.
    pub fn with_alpha(self, alpha: f32) -> Self {
        Self { a: alpha, ..self }
    }
}

/// One item of the draw list.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawItem {
    /// The element the item belongs to.
    pub id: ElementId,
    /// The animated bounds, in pixels.
    pub bounds: Rect,
    /// The corner radii of the shape.
    pub radii: Radii,
    /// The colour of the surface.
    pub color: Color,
    /// The animated opacity.
    pub opacity: f32,
    /// The order in which the item is drawn.
    pub z_index: i32,
    /// The clipping shape, when the element has one.
    pub clip: Option<RoundedRect>,
}

impl DrawItem {
    /// The effective alpha of the item.
    pub fn alpha(&self) -> f32 {
        (self.color.a * self.opacity).clamp(0.0, 1.0)
    }
}

/// The list of items to draw for one frame.
#[derive(Clone, Debug, Default)]
pub struct DrawList {
    /// The width of the viewport, in pixels.
    pub width: f32,
    /// The height of the viewport, in pixels.
    pub height: f32,
    /// The items, in draw order.
    pub items: Vec<DrawItem>,
}

impl DrawList {
    /// Creates an empty draw list for a viewport.
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            items: Vec::new(),
        }
    }

    /// Builds the draw list of a scene snapshot.
    pub fn from_snapshot(snapshot: &SceneSnapshot) -> Self {
        let mut list = Self::new(0.0, 0.0);

        for presentation in &snapshot.presentations {
            if presentation.opacity <= 0.0 {
                continue;
            }

            let (radii, clip) = match presentation.clip {
                Some(Shape::RoundedRect(rounded)) => (
                    rounded.radii,
                    Some(RoundedRect::new(presentation.bounds, rounded.radii)),
                ),
                _ => (Radii::NONE, None),
            };

            list.items.push(DrawItem {
                id: presentation.id,
                bounds: presentation.bounds,
                radii,
                color: Color::WHITE,
                opacity: presentation.opacity as f32,
                z_index: 0,
                clip,
            });
        }

        list
    }

    /// The number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether there is nothing to draw.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The total area that is covered by the items, in pixels².
    ///
    /// A rough measure of the work the GPU has to do; it is used by the
    /// benchmarks and by the invalidation logic.
    pub fn covered_area(&self) -> f64 {
        self.items
            .iter()
            .map(|item| item.bounds.area())
            .sum::<f64>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use weave_scene::Scene;

    #[test]
    fn an_empty_scene_produces_an_empty_draw_list() {
        let mut scene = Scene::new();
        let snapshot = scene.tick(0.0);
        let list = DrawList::from_snapshot(&snapshot);

        assert!(list.is_empty());
        assert_eq!(list.covered_area(), 0.0);
    }

    #[test]
    fn every_element_becomes_an_item() {
        let mut scene = Scene::new();
        scene.layout("surface", Rect::new(10.0, 20.0, 100.0, 40.0));
        let snapshot = scene.tick(0.0);

        let list = DrawList::from_snapshot(&snapshot);
        assert_eq!(list.len(), 1);
        assert_eq!(list.items[0].bounds, Rect::new(10.0, 20.0, 100.0, 40.0));
        assert_eq!(list.covered_area(), 4000.0);
    }

    #[test]
    fn transparent_elements_are_skipped() {
        let mut scene = Scene::new();
        scene.layout("surface", Rect::new(0.0, 0.0, 100.0, 40.0));
        scene.opacity("surface", 0.0);

        // The opacity fades out; once it reached zero the element is not
        // drawn any more.
        let mut list = DrawList::from_snapshot(&scene.tick(0.0));
        for frame in 1..30 {
            list = DrawList::from_snapshot(&scene.tick(frame as f64 / 60.0));
        }

        assert!(list.is_empty());
    }

    #[test]
    fn the_alpha_combines_the_colour_and_the_opacity() {
        let mut scene = Scene::new();
        scene.layout("surface", Rect::new(0.0, 0.0, 10.0, 10.0));
        let snapshot = scene.tick(0.0);

        let mut list = DrawList::from_snapshot(&snapshot);
        list.items[0].color = Color::new(1.0, 1.0, 1.0, 0.5);

        let item = &list.items[0];
        assert!((item.alpha() - 0.5).abs() < 1e-6);
    }
}
