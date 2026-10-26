//! The CSS a presentation state translates into.
//!
//! The style computation is deliberately separated from the DOM: it is pure
//! string building, so it can be tested on any target, and the DOM part
//! only has to apply the result.

use weave_geometry::Shape;
use weave_scene::Presentation;

/// The CSS declarations of one element.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DomStyle {
    /// The `left` declaration.
    pub left: String,
    /// The `top` declaration.
    pub top: String,
    /// The `width` declaration.
    pub width: String,
    /// The `height` declaration.
    pub height: String,
    /// The `opacity` declaration.
    pub opacity: String,
    /// The `clip-path` declaration, when the element has a shape.
    pub clip_path: Option<String>,
    /// The `transform` declaration, when the element moves with a FLIP
    /// transform.
    pub transform: Option<String>,
}

impl DomStyle {
    /// Computes the style of a presentation.
    pub fn for_presentation(presentation: &Presentation) -> Self {
        let bounds = &presentation.bounds;

        let mut style = Self {
            left: format!("{}px", bounds.x),
            top: format!("{}px", bounds.y),
            width: format!("{}px", bounds.width.max(0.0)),
            height: format!("{}px", bounds.height.max(0.0)),
            opacity: format!("{}", presentation.opacity.clamp(0.0, 1.0)),
            clip_path: None,
            transform: None,
        };

        if let Some(Shape::RoundedRect(rounded)) = presentation.clip {
            let radii = rounded.radii;
            style.clip_path = Some(format!(
                "inset(0px 0px 0px 0px round {}px {}px {}px {}px)",
                radii.top_left, radii.top_right, radii.bottom_right, radii.bottom_left
            ));
        }

        let transform = presentation.transform;
        if !transform.is_identity() {
            style.transform = Some(format!(
                "translate({}px, {}px) scale({}, {})",
                transform.translate_x, transform.translate_y, transform.scale_x, transform.scale_y
            ));
        }

        style
    }

    /// The declarations as a CSS text, for a stylesheet based renderer.
    pub fn to_css(&self) -> String {
        let mut declarations = vec![
            format!("left: {}", self.left),
            format!("top: {}", self.top),
            format!("width: {}", self.width),
            format!("height: {}", self.height),
            format!("opacity: {}", self.opacity),
        ];
        if let Some(clip_path) = &self.clip_path {
            declarations.push(format!("clip-path: {clip_path}"));
        }
        if let Some(transform) = &self.transform {
            declarations.push(format!("transform: {transform}"));
        }
        declarations.join("; ")
    }
}
