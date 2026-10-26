//! # weave-scene
//!
//! Stable element identity, FLIP (First, Last, Invert, Play) layout
//! transitions and choreography for the Weave engine.
//!
//! The scene layer *does not* own layout or input: applications describe
//! where elements should be, Weave animates the difference.
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

/// Version of the Weave engine this crate belongs to.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
