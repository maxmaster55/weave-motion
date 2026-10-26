//! Animated values: a spring-driven property of any interpolable type.

use crate::animatable::Animatable;
use crate::spring::SpringConfig;

/// A property that moves towards a destination using a spring.
///
/// This is the type applications actually use. It generalises
/// [`Spring`](crate::Spring) to vectors and any other [`Animatable`] type,
/// while keeping the same guarantees:
///
/// * the destination can be changed at any time without losing velocity,
/// * the state only depends on elapsed time, not on the frame rate,
/// * the value reports when it settled so schedulers can idle.
///
/// ```
/// use weave_core::{Animated, SpringConfig, Vec2};
///
/// let config = SpringConfig::unit_mass(300.0, 0.8)?;
/// let mut scale = Animated::new(Vec2::ONE, Vec2::splat(1.06), config);
///
/// scale.advance(1.0 / 60.0);
/// let velocity = scale.velocity();
///
/// scale.set_target(Vec2::ONE);
/// assert_eq!(scale.velocity(), velocity);
/// # Ok::<(), weave_core::SpringError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Animated<T: Animatable> {
    value: T,
    velocity: T,
    target: T,
    config: SpringConfig,
}

impl<T: Animatable> Animated<T> {
    /// Creates an animated value at `value`, moving towards `target`.
    pub fn new(value: T, target: T, config: SpringConfig) -> Self {
        Self {
            value,
            velocity: T::zero(),
            target,
            config,
        }
    }

    /// Creates an animated value at rest at `value`.
    pub fn resting(value: T, config: SpringConfig) -> Self {
        Self {
            value,
            velocity: T::zero(),
            target: value,
            config,
        }
    }

    /// Creates an animated value with an explicit initial velocity.
    pub fn with_velocity(value: T, velocity: T, target: T, config: SpringConfig) -> Self {
        Self {
            value,
            velocity,
            target,
            config,
        }
    }

    /// Current value.
    pub fn value(&self) -> T {
        self.value
    }

    /// Current velocity.
    pub fn velocity(&self) -> T {
        self.velocity
    }

    /// Current destination.
    pub fn target(&self) -> T {
        self.target
    }

    /// The spring configuration in use.
    pub fn config(&self) -> SpringConfig {
        self.config
    }

    /// Replaces the spring configuration, keeping the current state.
    pub fn set_config(&mut self, config: SpringConfig) {
        self.config = config;
    }

    /// Changes the destination, preserving the current value **and**
    /// velocity.
    pub fn set_target(&mut self, target: T) {
        self.target = target;
    }

    /// Overrides the velocity, for example when a gesture is released.
    pub fn set_velocity(&mut self, velocity: T) {
        self.velocity = velocity;
    }

    /// Jumps to `value` and cancels any motion.
    pub fn snap(&mut self, value: T) {
        self.value = value;
        self.target = value;
        self.velocity = T::zero();
    }

    /// Completes the animation immediately.
    pub fn settle(&mut self) {
        self.value = self.target;
        self.velocity = T::zero();
    }

    /// Returns `true` when the value reached its destination and stopped.
    pub fn is_settled(&self) -> bool {
        T::distance(self.value, self.target) <= self.config.position_epsilon
            && T::distance(self.velocity, T::zero()) <= self.config.velocity_epsilon
    }

    /// Returns `true` while the value still has motion left.
    pub fn is_active(&self) -> bool {
        !self.is_settled()
    }

    /// Advances the simulation by `dt` seconds.
    ///
    /// Returns `true` while the value is still moving.
    pub fn advance(&mut self, dt: f64) -> bool {
        if self.is_settled() {
            self.value = self.target;
            self.velocity = T::zero();
            return false;
        }
        if !dt.is_finite() || dt <= 0.0 {
            return true;
        }

        let (value, velocity) =
            T::integrate(self.value, self.velocity, self.target, &self.config, dt);
        self.value = value;
        self.velocity = velocity;

        if self.is_settled() {
            self.value = self.target;
            self.velocity = T::zero();
            return false;
        }
        true
    }
}
