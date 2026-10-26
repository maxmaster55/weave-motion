//! # weave-core
//!
//! Renderer-independent motion primitives for the Weave engine.
//!
//! This crate contains **no** renderer, windowing, or framework coupling:
//! it only knows about numbers, time and the laws of motion.
//!
//! Milestone 1 (see `DESIGN.md`) delivers the exact-time damped-spring
//! solver. The remaining modules (tokens, tweens, scheduler) land in the
//! following checkpoints.
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

/// Version of the Weave engine this crate belongs to.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
