//! Transforms and the FLIP technique.

use crate::id::ElementId;
use weave_core::{Animatable, ValueSnapshot};
use weave_geometry::Rect;

/// An affine transform restricted to translation and uniform-per-axis scale.
///
/// This is exactly what FLIP needs: the element is drawn at its old
/// position, scaled into its new box, and the transform is animated back to
/// the identity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    /// Horizontal translation, in pixels.
    pub translate_x: f64,
    /// Vertical translation, in pixels.
    pub translate_y: f64,
    /// Horizontal scale factor.
    pub scale_x: f64,
    /// Vertical scale factor.
    pub scale_y: f64,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    /// The transform that changes nothing.
    pub const IDENTITY: Self = Self {
        translate_x: 0.0,
        translate_y: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
    };

    /// Whether the transform is (close to) the identity.
    pub fn is_identity(&self) -> bool {
        self.translate_x == 0.0
            && self.translate_y == 0.0
            && self.scale_x == 1.0
            && self.scale_y == 1.0
    }

    /// The interpolated transform.
    pub fn lerp(&self, other: &Transform, t: f64) -> Transform {
        Transform {
            translate_x: self.translate_x + (other.translate_x - self.translate_x) * t,
            translate_y: self.translate_y + (other.translate_y - self.translate_y) * t,
            scale_x: self.scale_x + (other.scale_x - self.scale_x) * t,
            scale_y: self.scale_y + (other.scale_y - self.scale_y) * t,
        }
    }
}

impl Animatable for Transform {
    fn lerp(a: Self, b: Self, t: f64) -> Self {
        a.lerp(&b, t)
    }

    fn add_scaled(self, other: Self, scale: f64) -> Self {
        Transform {
            translate_x: self.translate_x + other.translate_x * scale,
            translate_y: self.translate_y + other.translate_y * scale,
            scale_x: self.scale_x + other.scale_x * scale,
            scale_y: self.scale_y + other.scale_y * scale,
        }
    }

    fn scale(self, factor: f64) -> Self {
        Transform {
            translate_x: self.translate_x * factor,
            translate_y: self.translate_y * factor,
            scale_x: self.scale_x * factor,
            scale_y: self.scale_y * factor,
        }
    }

    fn zero() -> Self {
        Transform {
            translate_x: 0.0,
            translate_y: 0.0,
            scale_x: 0.0,
            scale_y: 0.0,
        }
    }

    fn distance(a: Self, b: Self) -> f64 {
        (a.translate_x - b.translate_x)
            .abs()
            .max((a.translate_y - b.translate_y).abs())
            .max((a.scale_x - b.scale_x).abs())
            .max((a.scale_y - b.scale_y).abs())
    }

    fn integrate(
        value: Self,
        velocity: Self,
        target: Self,
        config: &weave_core::SpringConfig,
        dt: f64,
    ) -> (Self, Self) {
        let (translate_x, vx) = weave_core::solve(
            config,
            value.translate_x,
            velocity.translate_x,
            target.translate_x,
            dt,
        );
        let (translate_y, vy) = weave_core::solve(
            config,
            value.translate_y,
            velocity.translate_y,
            target.translate_y,
            dt,
        );
        let (scale_x, sx) =
            weave_core::solve(config, value.scale_x, velocity.scale_x, target.scale_x, dt);
        let (scale_y, sy) =
            weave_core::solve(config, value.scale_y, velocity.scale_y, target.scale_y, dt);

        (
            Transform {
                translate_x,
                translate_y,
                scale_x,
                scale_y,
            },
            Transform {
                translate_x: vx,
                translate_y: vy,
                scale_x: sx,
                scale_y: sy,
            },
        )
    }

    fn snapshot(self) -> ValueSnapshot {
        ValueSnapshot::Quad(weave_core::Vec4::new(
            self.translate_x,
            self.translate_y,
            self.scale_x,
            self.scale_y,
        ))
    }
}

/// The First-Last-Invert-Play description of a layout change.
///
/// * **First** — where the element was,
/// * **Last** — where the element is going to be,
/// * **Invert** — the transform that draws the first box at the last
///   position,
/// * **Play** — the identity transform, the end of the animation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flip {
    /// The bounds before the layout change.
    pub first: Rect,
    /// The bounds after the layout change.
    pub last: Rect,
}

impl Flip {
    /// Creates a FLIP description.
    pub fn new(first: Rect, last: Rect) -> Self {
        Self { first, last }
    }

    /// The point the transform scales around: the centre of the first box.
    pub fn origin(&self) -> (f64, f64) {
        (self.first.center_x(), self.first.center_y())
    }

    /// The transform that maps the first box onto the last box.
    pub fn invert(&self) -> Transform {
        let scale_x = if self.first.width.abs() > f64::EPSILON {
            self.last.width / self.first.width
        } else {
            1.0
        };
        let scale_y = if self.first.height.abs() > f64::EPSILON {
            self.last.height / self.first.height
        } else {
            1.0
        };

        let (origin_x, origin_y) = self.origin();

        Transform {
            translate_x: self.last.center_x() - origin_x,
            translate_y: self.last.center_y() - origin_y,
            scale_x,
            scale_y,
        }
    }

    /// The transform the animation ends with.
    pub fn play(&self) -> Transform {
        Transform::IDENTITY
    }
}

/// How an element moves to its new position.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TransitionMode {
    /// The bounds themselves are animated. The content re-flows, so text is
    /// never stretched. This is the default and the right choice for
    /// surfaces that change size.
    #[default]
    Bounds,
    /// The element keeps its size and a FLIP transform carries it from the
    /// old box to the new one. Use it for shared elements that move between
    /// screens without changing their content.
    Flip,
}

/// A transition that belongs to a specific element.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElementTransition {
    /// The element the transition belongs to.
    pub element: ElementId,
    /// The bounds before the change.
    pub first: Rect,
    /// The bounds after the change.
    pub last: Rect,
    /// How the element moves.
    pub mode: TransitionMode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_inverted_transform_maps_the_first_box_onto_the_last() {
        let flip = Flip::new(
            Rect::new(0.0, 0.0, 100.0, 50.0),
            Rect::new(200.0, 100.0, 300.0, 150.0),
        );
        let invert = flip.invert();

        let (origin_x, origin_y) = flip.origin();
        let mapped_x =
            origin_x + (flip.first.center_x() - origin_x) * invert.scale_x + invert.translate_x;
        let mapped_y =
            origin_y + (flip.first.center_y() - origin_y) * invert.scale_y + invert.translate_y;

        assert!((mapped_x - flip.last.center_x()).abs() < 1e-9);
        assert!((mapped_y - flip.last.center_y()).abs() < 1e-9);
        assert!((invert.scale_x - 3.0).abs() < 1e-9);
        assert!((invert.scale_y - 3.0).abs() < 1e-9);
    }

    #[test]
    fn a_flip_without_a_size_change_only_translates() {
        let flip = Flip::new(
            Rect::new(0.0, 0.0, 100.0, 50.0),
            Rect::new(40.0, 10.0, 100.0, 50.0),
        );
        let invert = flip.invert();

        assert_eq!(invert.scale_x, 1.0);
        assert_eq!(invert.scale_y, 1.0);
        assert_eq!(invert.translate_x, 40.0);
        assert_eq!(invert.translate_y, 10.0);
    }

    #[test]
    fn a_degenerate_first_box_does_not_produce_nan() {
        let flip = Flip::new(
            Rect::new(0.0, 0.0, 0.0, 0.0),
            Rect::new(10.0, 10.0, 50.0, 50.0),
        );
        let invert = flip.invert();
        assert!(invert.scale_x.is_finite());
        assert!(invert.scale_y.is_finite());
    }
}
