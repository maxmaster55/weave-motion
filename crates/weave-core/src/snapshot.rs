//! Renderer-agnostic snapshots of animated values.

use crate::math::abs;
use crate::types::{Vec2, Vec4};

/// A description of an animated value, independent of its Rust type.
///
/// Adapters (GPU, DOM, terminal) consume snapshots instead of reaching into
/// the animation state, which keeps the core decoupled from any renderer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ValueSnapshot {
    /// A single scalar.
    Scalar(f64),
    /// A two dimensional vector.
    Vector(Vec2),
    /// A four dimensional vector.
    Quad(Vec4),
}

impl ValueSnapshot {
    /// Returns the scalar payload, if this is a scalar.
    pub fn as_scalar(&self) -> Option<f64> {
        match *self {
            ValueSnapshot::Scalar(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the vector payload, if this is a vector.
    pub fn as_vector(&self) -> Option<Vec2> {
        match *self {
            ValueSnapshot::Vector(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the quad payload, if this is a quad.
    pub fn as_quad(&self) -> Option<Vec4> {
        match *self {
            ValueSnapshot::Quad(value) => Some(value),
            _ => None,
        }
    }

    /// The largest absolute component of the payload.
    pub fn magnitude(&self) -> f64 {
        match *self {
            ValueSnapshot::Scalar(value) => abs(value),
            ValueSnapshot::Vector(value) => abs(value.x).max(abs(value.y)),
            ValueSnapshot::Quad(value) => abs(value.x)
                .max(abs(value.y))
                .max(abs(value.z))
                .max(abs(value.w)),
        }
    }
}
