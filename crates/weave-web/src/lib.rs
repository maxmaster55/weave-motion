//! # weave-web
//!
//! A web adapter for the [Weave](../README.md) motion engine.
//!
//! Weave computes the motion; the adapter only *applies* it. The DOM
//! adapter therefore never uses the Web Animations API: the browser is
//! told where every element is, and the engine decides how it gets there.
//! That keeps interruption, reduced motion and frame pacing under the
//! control of the engine.
//!
//! ## Layers
//!
//! | Layer | Needs a browser | Contents |
//! |---|---|---|
//! | `weave_web::DomStyle` | no | the CSS of a presentation state |
//! | `weave_web::DomRenderer` | yes | applies the styles to the DOM |
//! | `weave_web::Motion` | no (exported to JS with `web`) | handle-based springs for JavaScript |
//!
//! `Motion` has a plain JavaScript twin in `js/weave.js`, so web apps can
//! use the wasm build or no wasm at all with the same API.
//!
//! The style computation is pure string building, so it is covered by
//! tests that run on any target.
//!
//! ```
//! use weave_scene::Scene;
//! use weave_web::DomStyle;
//!
//! let mut scene = Scene::new();
//! scene.layout("card", weave_geometry::Rect::new(10.0, 20.0, 100.0, 40.0));
//! let snapshot = scene.tick(0.0);
//!
//! let style = DomStyle::for_presentation(snapshot.get_by_key("card").unwrap());
//! assert_eq!(style.left, "10px");
//! assert_eq!(style.width, "100px");
//! ```

#![warn(missing_docs)]

extern crate alloc;

mod motion;
mod style;

pub use motion::{preset, Motion};
pub use style::DomStyle;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
mod dom;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use dom::DomRenderer;

/// Version of the Weave engine this crate belongs to.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
