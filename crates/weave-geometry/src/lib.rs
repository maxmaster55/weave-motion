//! # weave-geometry
//!
//! Normalized geometry types and shape morphing for the Weave engine.
//!
//! Shapes are described in a normalized coordinate space so that a morph
//! can be computed once and reused for any output bounds. Tessellation and
//! rasterization are deliberately **not** part of this crate: they are
//! adapter concerns.
//!
//! ## Contents
//!
//! | Module | Purpose |
//! |---|---|
//! | `rect` | axis aligned rectangles |
//! | `rounded` | rounded rectangles and corner radii |
//! | `shape` | shapes and the morphing rules between them |
//! | `point` | points and cubic Bézier segments |
//! | `path` | contours and paths |
//! | `correspondence` | matching the points of two contours |
//! | `morph` | morph plans between compatible paths |
//! | `cache` | caching of morph plans |
//!
//! ## Two levels of morphing
//!
//! * **Tier A — parametric.** [`RoundedRect`] interpolates its bounds and
//!   its corner radii. This covers most UI transitions and never breaks.
//! * **Tier B — contours.** [`MorphPlan`] matches two paths point by point
//!   (arc-length resampling, winding normalisation, start-point alignment)
//!   and interpolates the corresponding control points. A plan is refused
//!   when the result would twist or self-intersect.
//!
//! ## Safety rules
//!
//! Morphing must never produce an impossible shape. Three mechanisms
//! enforce that:
//!
//! 1. the morph progress is **clamped** to `[0, 1]` by default, so a
//!    spring overshoot cannot extrapolate a rounded rectangle;
//! 2. corner radii are **clamped** to the bounds of the rectangle after
//!    every interpolation;
//! 3. a contour morph is **refused** when the correspondence would produce
//!    a self-intersecting outline.
//!
//! ```
//! use weave_geometry::{Rect, Radii, Shape};
//!
//! let from = Shape::pill(Rect::new(0.0, 0.0, 120.0, 40.0));
//! let to = Shape::rounded_rect(Rect::new(0.0, 0.0, 320.0, 240.0), Radii::uniform(24.0));
//!
//! let morph = weave_geometry::Morph::new(from, to).unwrap();
//! let shape = morph.apply(0.5).unwrap();
//! assert_eq!(shape.bounds().width, 220.0);
//! ```
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

extern crate alloc;

mod cache;
mod correspondence;
mod morph;
mod path;
mod point;
mod rect;
mod rounded;
mod shape;

pub use cache::{MorphCache, MorphCacheKey};
pub use correspondence::{
    alignment_cost, best_shift, has_self_intersection, normalize_winding, resample, rotate,
    signed_area,
};
pub use morph::{MorphPlan, DEFAULT_SAMPLES};
pub use path::{Contour, Path};
pub use point::{CubicSegment, Point};
pub use rect::Rect;
pub use rounded::{Radii, RoundedRect};
pub use shape::{Extrapolation, Morph, Shape};

/// Version of the Weave engine this crate belongs to.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
