//! # weaver-motion
//!
//! Renderer-independent motion primitives for the Weave engine.
//!
//! This crate contains **no** renderer, windowing, or framework coupling:
//! it only knows about numbers, time and the laws of motion.
//!
//! ## Contents
//!
//! | Module | Purpose |
//! |---|---|
//! | [`spring`] | exact-time damped-spring solver |
//! | [`value`] | [`Animated<T>`]: a spring-driven property of any interpolable type |
//! | [`tween`] | fixed-duration tweens and easing curves |
//! | [`tokens`] | Weave's original named presets, superseded by roles |
//! | [`scheme`] | complete spring sets per motion role (standard, expressive, calm) |
//! | [`choreography`] | stagger timing for groups |
//! | [`clock`] | the shared monotonic clock and its gap policies |
//! | [`policy`] | accessibility policy (full / reduced / off motion), role-aware |
//! | [`scheduler`] | the animation registry and frame scheduler |
//! | [`engine`] | the application facing [`MotionEngine`] façade |
//! | [`snapshot`] | renderer-agnostic value snapshots |
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
//! The motion guide every product built on this crate follows (roles,
//! durations, live-data rules, reduced motion, accessibility limits) is
//! `docs/MOTION.md` in the repository.
//!
//! ```
//! use weaver_motion::{Animated, SpringConfig, Vec2};
//!
//! let config = SpringConfig::unit_mass(300.0, 0.8)?;
//! let mut scale = Animated::new(Vec2::ONE, Vec2::splat(1.06), config);
//!
//! scale.advance(1.0 / 60.0);
//! let velocity = scale.velocity();
//!
//! scale.set_target(Vec2::ONE);
//! assert_eq!(scale.velocity(), velocity);
//! # Ok::<(), weaver_motion::SpringError>(())
//! ```
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

extern crate alloc;

mod animatable;
mod choreography;
mod clock;
mod engine;
mod error;
mod math;
mod policy;
mod scheduler;
mod scheme;
mod snapshot;
mod spring;
mod tokens;
mod tween;
mod types;
mod value;

pub use animatable::Animatable;
pub use choreography::{
    stagger_delay, stagger_delays, stagger_interval, STAGGER_BUDGET, STAGGER_MAX_INTERVAL,
    STAGGER_MAX_ITEMS,
};
pub use clock::{MotionClock, Tick, TimeGapPolicy};
pub use engine::MotionEngine;
pub use error::SpringError;
pub use policy::{MotionPolicy, CROSSFADE_DISTANCE_PX};
pub use scheduler::{
    AnimatedValue, AnimationEvent, AnimationId, AnimationKind, AnimationSnapshot, MotionSnapshot,
    Scheduler,
};
pub use scheme::{MotionRole, MotionScheme, EFFECTS_EPSILON};
pub use snapshot::ValueSnapshot;
pub use spring::{
    solve, Spring, SpringConfig, DEFAULT_POSITION_EPSILON, DEFAULT_VELOCITY_EPSILON,
    MIN_DAMPING_RATIO, SETTLE_COLOR, SETTLE_OPACITY, SETTLE_POSITION_PX, SETTLE_SCALE,
};
pub use tokens::{MotionPreset, MotionToken};
pub use tween::{Easing, Tween, TweenConfig};
pub use types::{Vec2, Vec4};
pub use value::Animated;

/// Version of the Weave engine this crate belongs to.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
