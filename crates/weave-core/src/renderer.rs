//! The renderer contract.
//!
//! Adapters implement [`MotionRenderer`]: they receive an immutable
//! snapshot of the presentation state and draw it. The engine never pushes
//! imperative per-property mutations into a renderer, which keeps the
//! renderer replaceable and makes a frame reproducible.

use crate::scheduler::MotionSnapshot;

/// A renderer that can present the motion state of an application.
pub trait MotionRenderer {
    /// Draws the given snapshot.
    fn present(&mut self, snapshot: &MotionSnapshot);

    /// Asks the host to schedule another frame.
    fn schedule_frame(&mut self);
}
