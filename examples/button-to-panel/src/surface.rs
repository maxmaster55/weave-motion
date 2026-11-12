//! The demo scene: a pill button that expands into a panel.

use std::string::String;

use weave_demo_support::Canvas;
use weave_geometry::{Radii, Rect, Shape};
use weave_scene::{Cue, Direction, Gate, Scene, SceneSnapshot};
use weaver_motion::MotionPolicy;

/// The bounds of the button, in canvas cells.
pub const BUTTON: Rect = Rect::new(28.0, 9.0, 20.0, 4.0);

/// The bounds of the panel, in canvas cells.
pub const PANEL: Rect = Rect::new(8.0, 4.0, 60.0, 14.0);

/// The key of the surface element.
pub const SURFACE: &str = "surface";
/// The key of the icon element.
pub const ICON: &str = "icon";
/// The key of the label element.
pub const LABEL: &str = "label";

/// The pill radius of the button.
fn button_radii() -> Radii {
    Radii::uniform(BUTTON.height / 2.0)
}

/// The corner radius of the panel.
fn panel_radii() -> Radii {
    Radii::uniform(6.0)
}

/// The demo: a scene plus the state the demo needs.
pub struct Demo {
    scene: Scene,
    expanded: bool,
}

impl Default for Demo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo {
    /// Creates the demo in its collapsed state.
    pub fn new() -> Self {
        Self::with_policy(MotionPolicy::Full)
    }

    /// Creates the demo with an explicit motion policy.
    pub fn with_policy(policy: MotionPolicy) -> Self {
        let mut scene = Scene::with_policy(policy);

        // The surface: a pill that grows into a panel.
        scene.layout(SURFACE, BUTTON);
        scene.radii(SURFACE, button_radii());

        // The icon lives in the button, the label in the panel.
        scene.layout(
            ICON,
            Rect::new(BUTTON.center_x(), BUTTON.center_y(), 1.0, 1.0),
        );
        scene.set_content_visible(ICON, true);

        scene.layout(
            LABEL,
            Rect::new(PANEL.center_x(), PANEL.center_y(), 14.0, 1.0),
        );
        scene.opacity(LABEL, 0.0);
        scene.set_content_visible(LABEL, false);

        // Choreography: the icon fades out once the surface is wide enough,
        // the label appears a little later. On the way back the order is
        // reversed, so the label disappears before the surface gets small
        // again.
        let surface = scene.element_id(SURFACE).expect("the surface exists");
        let icon = scene.element_id(ICON).expect("the icon exists");
        let label = scene.element_id(LABEL).expect("the label exists");

        scene.cue(
            Gate::Progress {
                element: surface,
                threshold: 0.55,
                direction: Direction::Rising,
            },
            Cue::HideContent { element: icon },
        );
        scene.cue(
            Gate::Progress {
                element: surface,
                threshold: 0.45,
                direction: Direction::Falling,
            },
            Cue::RevealContent { element: icon },
        );
        scene.cue(
            Gate::Progress {
                element: surface,
                threshold: 0.5,
                direction: Direction::Rising,
            },
            Cue::RevealContent { element: label },
        );
        scene.cue(
            Gate::Progress {
                element: surface,
                threshold: 0.4,
                direction: Direction::Falling,
            },
            Cue::HideContent { element: label },
        );

        Self {
            scene,
            expanded: false,
        }
    }

    /// The scene behind the demo.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Whether the panel is currently the destination.
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// The active motion policy.
    pub fn policy(&self) -> MotionPolicy {
        self.scene.policy()
    }

    /// Switches the motion policy.
    pub fn set_policy(&mut self, policy: MotionPolicy) {
        self.scene.set_policy(policy);
    }

    /// Expands the button into the panel, or collapses it back.
    pub fn toggle(&mut self) {
        self.set_expanded(!self.expanded);
    }

    /// Expands or collapses the demo.
    pub fn set_expanded(&mut self, expanded: bool) {
        if expanded == self.expanded {
            return;
        }
        self.expanded = expanded;

        if expanded {
            self.scene.layout(SURFACE, PANEL);
            self.scene.radii(SURFACE, panel_radii());
        } else {
            self.scene.layout(SURFACE, BUTTON);
            self.scene.radii(SURFACE, button_radii());
        }
    }

    /// Hands a velocity to the surface, as a released drag would.
    pub fn fling(&mut self, velocity_x: f64, velocity_y: f64) {
        self.scene
            .set_bounds_velocity(SURFACE, Rect::new(velocity_x, velocity_y, 0.0, 0.0));
    }

    /// Advances the demo.
    pub fn tick(&mut self, timestamp: f64) -> SceneSnapshot {
        self.scene.tick(timestamp)
    }

    /// Draws the demo into a canvas.
    pub fn draw(&self, canvas: &mut Canvas) {
        canvas.clear();

        let surface = self.scene.presentation(SURFACE);
        let Some(surface) = surface else {
            return;
        };

        let bounds = surface.bounds;
        let radius = match surface.clip {
            Some(Shape::RoundedRect(rounded)) => rounded.radii.top_left,
            _ => 0.0,
        };

        // The surface: a dim fill with a crisp outline.
        canvas.rounded_rect_fill(
            bounds.x as isize,
            bounds.y as isize,
            bounds.width.max(1.0) as usize,
            bounds.height.max(1.0) as usize,
            radius.max(0.0) as usize,
            '.',
        );
        canvas.rounded_rect_outline(
            bounds.x as isize,
            bounds.y as isize,
            bounds.width.max(1.0) as usize,
            bounds.height.max(1.0) as usize,
            radius.max(0.0) as usize,
            1,
            '#',
        );

        // The content is clipped by the surface: nothing may be drawn
        // outside of it.
        let clip = weave_demo_support::ClipRect {
            x: bounds.x as isize,
            y: bounds.y as isize,
            width: bounds.width.max(1.0) as usize,
            height: bounds.height.max(1.0) as usize,
        };

        let icon = self.scene.presentation(ICON);
        if let Some(icon) = icon {
            if icon.content_visible && icon.opacity > 0.05 {
                canvas.set_clip(Some(clip));
                canvas.text(bounds.center_x() as isize, bounds.center_y() as isize, "+");
                canvas.set_clip(None);
            }
        }

        let label = self.scene.presentation(LABEL);
        if let Some(label) = label {
            if label.content_visible && label.opacity > 0.05 {
                canvas.set_clip(Some(clip));
                canvas.text(
                    bounds.center_x() as isize - 3,
                    bounds.center_y() as isize,
                    "Create",
                );
                canvas.set_clip(None);
            }
        }
    }

    /// A human readable report of the scene state.
    pub fn debug_report(&self) -> String {
        self.scene.debug_report()
    }

    /// A one line summary of the current state.
    pub fn status(&self) -> String {
        let surface = self.scene.presentation(SURFACE);
        let Some(surface) = surface else {
            return "no surface".to_string();
        };

        let radius = match surface.clip {
            Some(Shape::RoundedRect(rounded)) => rounded.radii.top_left,
            _ => 0.0,
        };

        format!(
            "{:<6} bounds={:.0}x{:.0}@{:.0},{:.0} radius={:.1} morph={:.2} opacity={:.2} {}",
            if self.expanded { "panel" } else { "button" },
            surface.bounds.width,
            surface.bounds.height,
            surface.bounds.x,
            surface.bounds.y,
            radius,
            surface.morph_progress,
            surface.opacity,
            if self.scene.is_idle() {
                "idle"
            } else {
                "running"
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::Demo;

    fn canvas_of(demo: &Demo) -> Canvas {
        let mut canvas = Canvas::new(76, 22);
        demo.draw(&mut canvas);
        canvas
    }

    #[test]
    fn the_button_is_drawn_as_a_pill() {
        let demo = Demo::new();
        let canvas = canvas_of(&demo);
        let rendered = canvas.render();

        // The pill is 20 cells wide and 4 cells high, centred at (38, 11).
        assert!(rendered.contains("#####"), "the pill outline is missing");
        assert!(rendered.contains('+'), "the icon is missing");
    }

    #[test]
    fn the_content_stays_inside_the_surface() {
        let mut demo = Demo::new();
        demo.set_expanded(true);
        for frame in 0..120 {
            demo.tick(frame as f64 / 60.0);
        }

        let canvas = canvas_of(&demo);
        let rendered = canvas.render();

        let bounds = demo.scene().animated_bounds(SURFACE).expect("bounds");
        for (y, line) in rendered.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ch == 'C' || ch == 'r' || ch == 'e' || ch == 'a' || ch == 't' {
                    assert!(
                        x as f64 >= bounds.x
                            && x as f64 <= bounds.x + bounds.width
                            && y as f64 >= bounds.y
                            && y as f64 <= bounds.y + bounds.height,
                        "the label leaked out of the surface at {x},{y}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_label_appears_only_when_the_surface_is_wide_enough() {
        let mut demo = Demo::new();
        demo.set_expanded(true);

        let mut revealed_while_narrow = false;
        for frame in 0..120 {
            demo.tick(frame as f64 / 60.0);

            let label = demo.scene().presentation(LABEL).expect("label");
            let surface = demo.scene().animated_bounds(SURFACE).expect("bounds");

            if label.content_visible && surface.width < crate::surface::PANEL.width * 0.5 {
                revealed_while_narrow = true;
            }
        }

        assert!(
            !revealed_while_narrow,
            "the label must not appear before the surface is wide enough"
        );
    }

    #[test]
    fn an_interruption_does_not_jump() {
        let mut demo = Demo::new();
        demo.set_expanded(true);
        for frame in 0..12 {
            demo.tick(frame as f64 / 60.0);
        }

        let before = demo.scene().animated_bounds(SURFACE).expect("bounds");
        let velocity_before = demo.scene().bounds_velocity(SURFACE).expect("velocity");

        demo.set_expanded(false);

        let after = demo.scene().animated_bounds(SURFACE).expect("bounds");
        let velocity_after = demo.scene().bounds_velocity(SURFACE).expect("velocity");

        assert_eq!(before.width, after.width);
        assert_eq!(velocity_before.width, velocity_after.width);
    }

    #[test]
    fn the_demo_goes_idle_after_settling() {
        let mut demo = Demo::new();
        demo.set_expanded(true);

        let mut frames = 0;
        while frames < 600 {
            frames += 1;
            demo.tick(frames as f64 / 60.0);
            if demo.scene().is_idle() {
                break;
            }
        }

        assert!(demo.scene().is_idle());
        assert!(!demo.scene().needs_frame());

        let bounds = demo.scene().animated_bounds(SURFACE).expect("bounds");
        assert!((bounds.width - PANEL.width).abs() < 0.01);
        assert!((bounds.height - PANEL.height).abs() < 0.01);
    }

    #[test]
    fn a_fling_hands_the_velocity_to_the_spring() {
        let mut demo = Demo::new();
        demo.fling(120.0, 0.0);

        let velocity = demo.scene().bounds_velocity(SURFACE).expect("velocity");
        assert!((velocity.x - 120.0).abs() < 1e-9, "{velocity:?}");
    }
}
