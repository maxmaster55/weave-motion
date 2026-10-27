//! Rounded rectangles and the corner radii they carry.

use crate::rect::Rect;
use weaver_motion::Animatable;

/// The corner radii of a rounded rectangle.
///
/// The order matches the CSS convention: top-left, top-right,
/// bottom-right, bottom-left.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Radii {
    /// Radius of the top-left corner.
    pub top_left: f64,
    /// Radius of the top-right corner.
    pub top_right: f64,
    /// Radius of the bottom-right corner.
    pub bottom_right: f64,
    /// Radius of the bottom-left corner.
    pub bottom_left: f64,
}

impl Radii {
    /// No rounding at all.
    pub const NONE: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    /// The same radius on every corner.
    pub const fn uniform(radius: f64) -> Self {
        Self::new(radius, radius, radius, radius)
    }

    /// Creates radii from the four corner values.
    pub const fn new(top_left: f64, top_right: f64, bottom_right: f64, bottom_left: f64) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    /// The largest radius of the four corners.
    pub fn max(&self) -> f64 {
        self.top_left
            .max(self.top_right)
            .max(self.bottom_right)
            .max(self.bottom_left)
    }

    /// Interpolates between two sets of radii.
    pub fn lerp(&self, other: &Radii, t: f64) -> Radii {
        Radii {
            top_left: self.top_left + (other.top_left - self.top_left) * t,
            top_right: self.top_right + (other.top_right - self.top_right) * t,
            bottom_right: self.bottom_right + (other.bottom_right - self.bottom_right) * t,
            bottom_left: self.bottom_left + (other.bottom_left - self.bottom_left) * t,
        }
    }

    /// Clamps the radii so that they describe a valid shape inside `rect`.
    ///
    /// The rules are the ones used by CSS border radii:
    ///
    /// * a radius never exceeds half of the rectangle's width or height,
    /// * the two radii of an edge never add up to more than the edge length.
    pub fn clamped(&self, rect: &Rect) -> Radii {
        let width = rect.width.max(0.0);
        let height = rect.height.max(0.0);

        let mut radii = *self;

        // A radius can never be larger than half of the smaller side.
        let limit = (width / 2.0).min(height / 2.0);
        radii = Radii {
            top_left: radii.top_left.clamp(0.0, limit),
            top_right: radii.top_right.clamp(0.0, limit),
            bottom_right: radii.bottom_right.clamp(0.0, limit),
            bottom_left: radii.bottom_left.clamp(0.0, limit),
        };

        // Horizontal edges: the two radii must fit into the width.
        if radii.top_left + radii.top_right > width {
            let sum = radii.top_left + radii.top_right;
            let scale = if sum > 0.0 { width / sum } else { 0.0 };
            radii.top_left *= scale;
            radii.top_right *= scale;
        }
        if radii.bottom_left + radii.bottom_right > width {
            let sum = radii.bottom_left + radii.bottom_right;
            let scale = if sum > 0.0 { width / sum } else { 0.0 };
            radii.bottom_left *= scale;
            radii.bottom_right *= scale;
        }

        // Vertical edges: the two radii must fit into the height.
        if radii.top_left + radii.bottom_left > height {
            let sum = radii.top_left + radii.bottom_left;
            let scale = if sum > 0.0 { height / sum } else { 0.0 };
            radii.top_left *= scale;
            radii.bottom_left *= scale;
        }
        if radii.top_right + radii.bottom_right > height {
            let sum = radii.top_right + radii.bottom_right;
            let scale = if sum > 0.0 { height / sum } else { 0.0 };
            radii.top_right *= scale;
            radii.bottom_right *= scale;
        }

        radii
    }
}

/// A rectangle with rounded corners.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RoundedRect {
    /// The bounding box of the shape.
    pub rect: Rect,
    /// The corner radii.
    pub radii: Radii,
}

impl RoundedRect {
    /// Creates a rounded rectangle, clamping the radii so that they stay
    /// valid for the given bounds.
    pub fn new(rect: Rect, radii: Radii) -> Self {
        Self {
            rect,
            radii: radii.clamped(&rect),
        }
    }

    /// Creates a rounded rectangle with the same radius on every corner.
    pub fn uniform(rect: Rect, radius: f64) -> Self {
        Self::new(rect, Radii::uniform(radius))
    }

    /// Creates a pill: a rounded rectangle whose radius is half its height.
    pub fn pill(rect: Rect) -> Self {
        Self::new(rect, Radii::uniform(rect.height / 2.0))
    }

    /// Interpolates between two rounded rectangles.
    ///
    /// The result is always a valid shape: the radii are clamped again after
    /// the interpolation, so an overshooting morph parameter can never
    /// produce an impossible geometry.
    pub fn lerp(&self, other: &RoundedRect, t: f64) -> RoundedRect {
        let rect = Animatable::lerp(self.rect, other.rect, t);
        let radii = self.radii.lerp(&other.radii, t).clamped(&rect);
        RoundedRect { rect, radii }
    }

    /// Scales the shape around the origin of its bounding box.
    pub fn scaled(&self, sx: f64, sy: f64) -> RoundedRect {
        RoundedRect {
            rect: self.rect.scaled(sx, sy),
            radii: Radii {
                top_left: self.radii.top_left * sx.min(sy),
                top_right: self.radii.top_right * sx.min(sy),
                bottom_right: self.radii.bottom_right * sx.min(sy),
                bottom_left: self.radii.bottom_left * sx.min(sy),
            },
        }
    }

    /// Whether the shape describes a drawable area.
    pub fn is_valid(&self) -> bool {
        self.rect.is_valid() && self.radii.max() >= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_radius_never_exceeds_half_the_side() {
        let rect = Rect::new(0.0, 0.0, 100.0, 40.0);
        let radii = Radii::uniform(80.0).clamped(&rect);
        assert_eq!(radii.max(), 20.0);
    }

    #[test]
    fn two_radii_never_exceed_their_edge() {
        let rect = Rect::new(0.0, 0.0, 100.0, 40.0);
        // 40 + 40 = 80 fits into the width of 100.
        let radii = Radii::uniform(40.0).clamped(&rect);
        assert!(radii.top_left + radii.top_right <= rect.width);

        // A narrow rectangle forces the radii down.
        let narrow = Rect::new(0.0, 0.0, 30.0, 40.0);
        let radii = Radii::uniform(40.0).clamped(&narrow);
        assert!(radii.top_left + radii.top_right <= narrow.width);
        assert!(radii.top_left + radii.bottom_left <= narrow.height);
    }

    #[test]
    fn interpolation_keeps_the_shape_valid() {
        let from = RoundedRect::pill(Rect::new(0.0, 0.0, 80.0, 32.0));
        let to = RoundedRect::uniform(Rect::new(0.0, 0.0, 320.0, 240.0), 24.0);

        for step in 0..=20 {
            let t = step as f64 / 20.0;
            let middle = from.lerp(&to, t);
            assert!(middle.is_valid());
            assert!(middle.radii.top_left + middle.radii.top_right <= middle.rect.width);
            assert!(middle.radii.top_left + middle.radii.bottom_left <= middle.rect.height);
        }
    }

    #[test]
    fn an_overshooting_parameter_cannot_break_the_shape() {
        let from = RoundedRect::pill(Rect::new(0.0, 0.0, 80.0, 32.0));
        let to = RoundedRect::uniform(Rect::new(0.0, 0.0, 320.0, 240.0), 24.0);

        // A spring overshoot would pass 1.0; the clamped radii keep the
        // result valid anyway.
        let overshoot = from.lerp(&to, 1.4);
        assert!(overshoot.is_valid());
    }

    #[test]
    fn a_pill_uses_half_the_height_as_radius() {
        let pill = RoundedRect::pill(Rect::new(0.0, 0.0, 120.0, 40.0));
        assert_eq!(pill.radii.top_left, 20.0);
    }
}
