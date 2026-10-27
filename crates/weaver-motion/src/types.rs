//! Small vector types used by the motion primitives.
//!
//! Weave uses `f64` internally: animation state has to survive long
//! interruption chains (a value may be retargeted dozens of times before it
//! settles) without accumulating error. Adapters convert to whatever their
//! renderer prefers (`f32` for GPU uploads) at the very last step.

use crate::math;

/// A two dimensional vector.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    /// Horizontal component.
    pub x: f64,
    /// Vertical component.
    pub y: f64,
}

impl Vec2 {
    /// The zero vector.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    /// The vector `(1, 1)`.
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    /// Creates a vector from its components.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Creates a vector with both components set to `value`.
    pub const fn splat(value: f64) -> Self {
        Self { x: value, y: value }
    }

    /// Euclidean length.
    pub fn length(self) -> f64 {
        math::sqrt(self.x * self.x + self.y * self.y)
    }

    /// Dot product.
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Converts to `f32` components, for GPU uploads.
    pub fn to_f32(self) -> [f32; 2] {
        [self.x as f32, self.y as f32]
    }
}

/// A four dimensional vector.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec4 {
    /// First component.
    pub x: f64,
    /// Second component.
    pub y: f64,
    /// Third component.
    pub z: f64,
    /// Fourth component.
    pub w: f64,
}

impl Vec4 {
    /// The zero vector.
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };

    /// Creates a vector from its components.
    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    /// Converts to `f32` components, for GPU uploads.
    pub fn to_f32(self) -> [f32; 4] {
        [self.x as f32, self.y as f32, self.z as f32, self.w as f32]
    }
}
