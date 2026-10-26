//! # weave-geometry
//!
//! Normalized geometry types and shape morphing for the Weave engine.
//!
//! Shapes are described in a normalized coordinate space so that a morph
//! can be computed once and reused for any output bounds. Tessellation and
//! rasterization are deliberately **not** part of this crate: they are
//! adapter concerns.
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

/// Version of the Weave engine this crate belongs to.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
