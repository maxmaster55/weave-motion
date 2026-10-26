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
mod id;
mod scene;
mod transform;

pub use choreography::{Choreographer, Cue, Gate};
pub use id::{ElementId, ElementRegistry};
pub use scene::{Presentation, Scene, SceneSnapshot};
pub use transform::{ElementTransition, Flip, Transform, TransitionMode};

/// Version of the Weave engine this crate belongs to.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
