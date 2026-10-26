//! Shapes and the morphing rules between them.

use crate::rect::Rect;
use crate::rounded::{Radii, RoundedRect};

/// A shape that Weave knows how to animate into another shape.
///
/// Only shapes with a guaranteed correspondence can be morphed. When two
/// shapes cannot be matched safely the morph is refused and the caller
/// falls back to a crossfade — a broken outline is worse than a fade.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// A rounded rectangle.
    RoundedRect(RoundedRect),
}

impl Shape {
    /// Creates a rounded rectangle shape.
    pub fn rounded_rect(rect: Rect, radii: Radii) -> Self {
        Shape::RoundedRect(RoundedRect::new(rect, radii))
    }

    /// Creates a pill shape.
    pub fn pill(rect: Rect) -> Self {
        Shape::RoundedRect(RoundedRect::pill(rect))
    }

    /// The bounding box of the shape.
    pub fn bounds(&self) -> Rect {
        match self {
            Shape::RoundedRect(rounded) => rounded.rect,
        }
    }

    /// Whether the shape can be morphed into `other`.
    pub fn is_compatible_with(&self, other: &Shape) -> bool {
        matches!(
            (self, other),
            (Shape::RoundedRect(_), Shape::RoundedRect(_))
        )
    }

    /// Interpolates towards `other`.
    ///
    /// Returns `None` when the shapes cannot be morphed safely.
    pub fn lerp(&self, other: &Shape, t: f64) -> Option<Shape> {
        match (self, other) {
            (Shape::RoundedRect(from), Shape::RoundedRect(to)) => {
                Some(Shape::RoundedRect(from.lerp(to, t)))
            }
            // Different shape families cannot be morphed into each other.
            #[allow(unreachable_patterns)]
            _ => None,
        }
    }
}

/// How a morph reacts to a progress value outside `[0, 1]`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Extrapolation {
    /// Clamp the progress: the shape never goes past its destination.
    ///
    /// This is the default. A spring overshoot must not turn a rounded
    /// rectangle into an impossible shape.
    #[default]
    Clamp,
    /// Let the progress exceed the range and extrapolate the geometry.
    ///
    /// Only safe for shapes whose geometry stays valid when extrapolated.
    Extrapolate,
}

/// A morph between two compatible shapes.
///
/// The morph owns the correspondence between the shapes; the caller only
/// has to feed it a progress value, usually driven by a spring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Morph {
    from: Shape,
    to: Shape,
    extrapolation: Extrapolation,
}

impl Morph {
    /// Creates a morph between two shapes.
    ///
    /// Returns `None` when the shapes are not compatible; the caller should
    /// then crossfade instead.
    pub fn new(from: Shape, to: Shape) -> Option<Self> {
        if !from.is_compatible_with(&to) {
            return None;
        }
        Some(Self {
            from,
            to,
            extrapolation: Extrapolation::default(),
        })
    }

    /// Allows the progress to exceed `[0, 1]`.
    pub fn with_extrapolation(mut self, extrapolation: Extrapolation) -> Self {
        self.extrapolation = extrapolation;
        self
    }

    /// The source shape.
    pub fn from(&self) -> Shape {
        self.from
    }

    /// The destination shape.
    pub fn to(&self) -> Shape {
        self.to
    }

    /// The progress range the morph accepts.
    pub fn extrapolation(&self) -> Extrapolation {
        self.extrapolation
    }

    /// Evaluates the morph at `progress`.
    ///
    /// With [`Extrapolation::Clamp`] (the default) the progress is clamped
    /// to `[0, 1]`, so an overshooting spring cannot produce invalid
    /// geometry.
    pub fn apply(&self, progress: f64) -> Option<Shape> {
        let progress = match self.extrapolation {
            Extrapolation::Clamp => progress.clamp(0.0, 1.0),
            Extrapolation::Extrapolate => progress,
        };
        self.from.lerp(&self.to, progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounded_rectangles_morph_into_each_other() {
        let from = Shape::pill(Rect::new(0.0, 0.0, 120.0, 40.0));
        let to = Shape::rounded_rect(Rect::new(0.0, 0.0, 320.0, 240.0), Radii::uniform(24.0));

        let morph = Morph::new(from, to).expect("compatible shapes");
        let shape = morph.apply(0.5).expect("morph result");

        let bounds = shape.bounds();
        assert_eq!(bounds.width, 220.0);
        assert_eq!(bounds.height, 140.0);
    }

    #[test]
    fn an_overshooting_progress_is_clamped() {
        let from = Shape::pill(Rect::new(0.0, 0.0, 120.0, 40.0));
        let to = Shape::rounded_rect(Rect::new(0.0, 0.0, 320.0, 240.0), Radii::uniform(24.0));

        let morph = Morph::new(from, to).unwrap();
        let clamped = morph.apply(1.4).unwrap();
        let end = morph.apply(1.0).unwrap();

        assert_eq!(clamped, end);
    }

    #[test]
    fn extrapolation_can_be_enabled_explicitly() {
        let from = Shape::pill(Rect::new(0.0, 0.0, 120.0, 40.0));
        let to = Shape::rounded_rect(Rect::new(0.0, 0.0, 320.0, 240.0), Radii::uniform(24.0));

        let morph = Morph::new(from, to)
            .unwrap()
            .with_extrapolation(Extrapolation::Extrapolate);
        let shape = morph.apply(1.2).unwrap();

        assert!(shape.bounds().width > 320.0);
    }
}
