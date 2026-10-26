//! # weave-scene
//!
//! Stable element identity, FLIP (First, Last, Invert, Play) layout
//! transitions and choreography for the Weave engine.
//!
//! The scene layer *does not* own layout or input: applications describe
//! where elements should be, Weave animates the difference.
//!
//! ## Contents
//!
//! | Module | Purpose |
//! |---|---|
//! | [`id`] | stable element identity ([`ElementId`], [`ElementRegistry`]) |
//! | [`transform`] | FLIP transforms and transition modes |
//! | [`choreography`] | dependencies between the parts of a transition |
//! | [`debug`] | human readable descriptions of the scene state |
//! | [`scene`] | the [`Scene`] itself |
//!
//! ## Rules
//!
//! * **Identity is stable.** Elements are identified by a key the
//!   application chooses, never by their position in a tree.
//! * **Destinations are separate from presentation.** The application
//!   declares where things should be; the scene animates how they get
//!   there.
//! * **Interruption is normal.** Retargeting an element keeps the current
//!   value and velocity, so a transition that is interrupted mid-flight
//!   continues instead of restarting.
//! * **Dependencies, not delays.** Choreography expresses *when* a part of
//!   a transition may start as a condition on the animated state, never as
//!   a fixed number of frames.
//!
//! ```
//! use weave_geometry::Rect;
//! use weave_scene::Scene;
//!
//! let mut scene = Scene::new();
//! scene.layout("button", Rect::new(0.0, 0.0, 120.0, 40.0));
//!
//! let snapshot = scene.tick(0.0);
//! let presentation = snapshot.get_by_key("button").unwrap();
//! assert_eq!(presentation.bounds.width, 120.0);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

mod choreography;
mod debug;
mod id;
mod scene;
mod transform;

pub use choreography::{Choreographer, Cue, Direction, Gate};
pub use debug::{describe as describe_snapshot, report as debug_report, DebugEntry};
pub use id::{ElementId, ElementRegistry};
pub use scene::{Presentation, Scene, SceneSnapshot};
pub use transform::{ElementTransition, Flip, Transform, TransitionMode};

/// Version of the Weave engine this crate belongs to.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The contract every rendering adapter implements.
///
/// Adapters receive an immutable snapshot of the presentation state and draw
/// it. The engine never pushes imperative per-property mutations into a
/// renderer, which keeps the renderer replaceable and makes a frame
/// reproducible.
pub trait MotionRenderer {
    /// Draws the given snapshot.
    fn present(&mut self, snapshot: &SceneSnapshot);
    /// Asks the host to schedule another frame.
    fn schedule_frame(&mut self);
}
