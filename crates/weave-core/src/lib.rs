//! # weave-core
//!
//! Renderer-independent motion primitives for the Weave engine.
//!
//! This crate contains **no** renderer, windowing, or framework coupling:
//! it only knows about numbers, time and the laws of motion.
//!
//! ## Contents
//!
//! * [`spring`] — the exact-time damped-spring solver at the heart of Weave.
//! * [`Spring`] / [`SpringConfig`] — a scalar value driven by that solver.
//!
//! ## Design rules
//!
//! 1. **Retargetable.** Changing a destination never discards the current
//!    velocity, so interrupted animations continue instead of restarting.
//! 2. **Exact-time physics.** Springs are evaluated in closed form, so the
//!    result depends only on the timestamps, never on the frame rate.
//! 3. **Idle at rest.** Values report when they settled so schedulers can
//!    stop requesting frames.
//! 4. **Portable.** The crate has no third-party dependencies in `std`
//!    builds, works on `no_std` + `alloc`, and contains no `unsafe` code.
//!
//! ```
//! use weave_core::{Spring, SpringConfig};
//!
//! let config = SpringConfig::unit_mass(300.0, 0.8)?;
//! let mut spring = Spring::new(0.0, 1.0, config);
//!
//! while spring.advance(1.0 / 60.0) {
//!     // ... render `spring.value()`
//! }
//! assert!(spring.is_settled());
//! # Ok::<(), weave_core::SpringError>(())
//! ```
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

mod error;
mod math;
mod spring;

pub use error::SpringError;
pub use spring::{solve, Spring, SpringConfig, DEFAULT_POSITION_EPSILON, DEFAULT_VELOCITY_EPSILON};

/// Version of the Weave engine this crate belongs to.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
