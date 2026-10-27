//! Interpolation traits shared by springs and tweens.

use crate::types::{Vec2, Vec4};

/// A value that can be interpolated, scaled and integrated by the motion
/// primitives.
///
/// Implementations exist for the scalar types and for the small vector types
/// used by the presentation state. The trait is intentionally tiny so that
/// applications can add their own types (colors, quaternions, …) without
/// touching the engine.
///
/// ```
/// use weaver_motion::Animatable;
///
/// let a = 0.0_f64;
/// let b = 10.0_f64;
/// assert_eq!(Animatable::lerp(a, b, 0.25), 2.5);
/// ```
pub trait Animatable: Copy + Clone + PartialEq + core::fmt::Debug + 'static {
    /// Interpolates between `a` and `b`.
    fn lerp(a: Self, b: Self, t: f64) -> Self;

    /// Returns `self + other * scale`.
    fn add_scaled(self, other: Self, scale: f64) -> Self;

    /// Returns `self * factor`.
    fn scale(self, factor: f64) -> Self;

    /// The additive identity.
    fn zero() -> Self;

    /// The largest absolute difference between two values, component wise.
    ///
    /// Used for settling tests, so it must be a metric that is zero only
    /// when the values are equal.
    fn distance(a: Self, b: Self) -> f64;

    /// Integrates the spring model for `dt` seconds.
    ///
    /// Returns the new `(value, velocity)`.
    fn integrate(
        value: Self,
        velocity: Self,
        target: Self,
        config: &crate::SpringConfig,
        dt: f64,
    ) -> (Self, Self);

    /// A renderer-agnostic description of the value.
    fn snapshot(self) -> crate::ValueSnapshot;
}

impl Animatable for f64 {
    fn lerp(a: Self, b: Self, t: f64) -> Self {
        a + (b - a) * t
    }

    fn add_scaled(self, other: Self, scale: f64) -> Self {
        self + other * scale
    }

    fn scale(self, factor: f64) -> Self {
        self * factor
    }

    fn zero() -> Self {
        0.0
    }

    fn distance(a: Self, b: Self) -> f64 {
        crate::math::abs(a - b)
    }

    fn integrate(
        value: Self,
        velocity: Self,
        target: Self,
        config: &crate::SpringConfig,
        dt: f64,
    ) -> (Self, Self) {
        crate::solve(config, value, velocity, target, dt)
    }

    fn snapshot(self) -> crate::ValueSnapshot {
        crate::ValueSnapshot::Scalar(self)
    }
}

impl Animatable for Vec2 {
    fn lerp(a: Self, b: Self, t: f64) -> Self {
        Self {
            x: a.x + (b.x - a.x) * t,
            y: a.y + (b.y - a.y) * t,
        }
    }

    fn add_scaled(self, other: Self, scale: f64) -> Self {
        Self {
            x: self.x + other.x * scale,
            y: self.y + other.y * scale,
        }
    }

    fn scale(self, factor: f64) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }

    fn zero() -> Self {
        Self::ZERO
    }

    fn distance(a: Self, b: Self) -> f64 {
        let dx = crate::math::abs(a.x - b.x);
        let dy = crate::math::abs(a.y - b.y);
        if dx > dy {
            dx
        } else {
            dy
        }
    }

    fn integrate(
        value: Self,
        velocity: Self,
        target: Self,
        config: &crate::SpringConfig,
        dt: f64,
    ) -> (Self, Self) {
        let (x, vx) = crate::solve(config, value.x, velocity.x, target.x, dt);
        let (y, vy) = crate::solve(config, value.y, velocity.y, target.y, dt);
        (Self { x, y }, Self { x: vx, y: vy })
    }

    fn snapshot(self) -> crate::ValueSnapshot {
        crate::ValueSnapshot::Vector(self)
    }
}

impl Animatable for Vec4 {
    fn lerp(a: Self, b: Self, t: f64) -> Self {
        Self {
            x: a.x + (b.x - a.x) * t,
            y: a.y + (b.y - a.y) * t,
            z: a.z + (b.z - a.z) * t,
            w: a.w + (b.w - a.w) * t,
        }
    }

    fn add_scaled(self, other: Self, scale: f64) -> Self {
        Self {
            x: self.x + other.x * scale,
            y: self.y + other.y * scale,
            z: self.z + other.z * scale,
            w: self.w + other.w * scale,
        }
    }

    fn scale(self, factor: f64) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
            w: self.w * factor,
        }
    }

    fn zero() -> Self {
        Self::ZERO
    }

    fn distance(a: Self, b: Self) -> f64 {
        let dx = crate::math::abs(a.x - b.x);
        let dy = crate::math::abs(a.y - b.y);
        let dz = crate::math::abs(a.z - b.z);
        let dw = crate::math::abs(a.w - b.w);
        let mut max = dx;
        if dy > max {
            max = dy;
        }
        if dz > max {
            max = dz;
        }
        if dw > max {
            max = dw;
        }
        max
    }

    fn integrate(
        value: Self,
        velocity: Self,
        target: Self,
        config: &crate::SpringConfig,
        dt: f64,
    ) -> (Self, Self) {
        let (x, vx) = crate::solve(config, value.x, velocity.x, target.x, dt);
        let (y, vy) = crate::solve(config, value.y, velocity.y, target.y, dt);
        let (z, vz) = crate::solve(config, value.z, velocity.z, target.z, dt);
        let (w, vw) = crate::solve(config, value.w, velocity.w, target.w, dt);
        (
            Self { x, y, z, w },
            Self {
                x: vx,
                y: vy,
                z: vz,
                w: vw,
            },
        )
    }

    fn snapshot(self) -> crate::ValueSnapshot {
        crate::ValueSnapshot::Quad(self)
    }
}
