//! Exact-time damped spring solver.
//!
//! Weave models every physical value as a damped harmonic oscillator
//!
//! ```text
//! m x'' + c x' + k (x - target) = 0
//! ```
//!
//! with `c = 2 * zeta * sqrt(k * m)`. The solver evaluates the **closed-form**
//! solution of that equation for the elapsed interval, so the result depends
//! only on the timestamps handed to it — never on how many frames were
//! dropped in between. This is what makes Weave animations reproducible on a
//! 30 Hz laptop, a 120 Hz display and after a one-second application
//! suspension alike.
//!
//! Retargeting is a first-class operation: [`Spring::set_target`] changes the
//! destination while keeping the current position **and** the current
//! velocity, so an interrupted animation continues smoothly instead of
//! restarting from zero.
//!
//! ## Parameter styles
//!
//! Configurations store stiffness `k` and damping ratio `zeta` (at unit mass
//! in practice). [`SpringConfig::from_duration_bounce`] accepts Apple's
//! design-facing *duration* and *bounce* instead, and the accessors
//! [`SpringConfig::perceptual_duration`], [`SpringConfig::bounce`],
//! [`SpringConfig::overshoot`] and [`SpringConfig::settle_time`] describe any
//! configuration in perceptual terms. `docs/MOTION.md` lists the conversions
//! to other libraries.
//!
//! ## Settle thresholds
//!
//! "Arrived" is a per-unit, perceptual judgement. Use
//! [`SpringConfig::with_settle_threshold`] with one of
//! [`SETTLE_POSITION_PX`], [`SETTLE_SCALE`], [`SETTLE_OPACITY`] or
//! [`SETTLE_COLOR`] for the property being animated.

use crate::error::SpringError;
use crate::math;

/// Tolerance used to decide whether the damping ratio is "critical".
///
/// Below `1 - CRITICAL_TOLERANCE` the solution is underdamped, above
/// `1 + CRITICAL_TOLERANCE` it is overdamped. In between, the critically
/// damped closed form is used: it stays numerically stable where the
/// underdamped form would divide by a vanishing `omega_d`.
const CRITICAL_TOLERANCE: f64 = 1e-4;

/// Default settling tolerance for position (in the unit of the animated
/// property).
pub const DEFAULT_POSITION_EPSILON: f64 = 0.01;

/// Default settling tolerance for velocity (in units per second).
pub const DEFAULT_VELOCITY_EPSILON: f64 = 0.01;

/// Smallest damping ratio the solver evaluates.
///
/// A spring with `zeta` below this value is evaluated as if it had exactly
/// this damping ratio. Without a floor, `zeta = 0` would oscillate forever
/// and keep a scheduler awake. 0.05 is the floor used by Motion (Framer
/// Motion); at that ratio a spring overshoots by about 85% and is far
/// outside any UI use, so the clamp only guards against misconfiguration.
/// The stored [`SpringConfig::damping_ratio`] is left as given; see
/// [`SpringConfig::effective_damping_ratio`].
pub const MIN_DAMPING_RATIO: f64 = 0.05;

/// Settle threshold for positions and sizes in pixels: half a device pixel.
///
/// Snapping by at most half a pixel is invisible. Motion (Framer Motion)
/// uses the same 0.5 rest delta for pixel-scale values.
pub const SETTLE_POSITION_PX: f64 = 0.5;

/// Settle threshold for scale factors (1.0 = natural size).
///
/// 0.001 of a 500 px element is half a pixel. Coarser values such as 0.01
/// are visible on large surfaces (4 px on a 400 px panel).
pub const SETTLE_SCALE: f64 = 0.001;

/// Settle threshold for opacity in `0..=1`: one step of an 8-bit channel.
pub const SETTLE_OPACITY: f64 = 1.0 / 255.0;

/// Settle threshold for a normalised colour channel in `0..=1`: one step of
/// an 8-bit channel.
pub const SETTLE_COLOR: f64 = 1.0 / 255.0;

/// Physical parameters of a damped harmonic oscillator.
///
/// ```
/// use weaver_motion::SpringConfig;
///
/// let config = SpringConfig::new(1.0, 300.0, 0.8).unwrap();
/// assert!(config.omega() > 0.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringConfig {
    /// Mass of the animated property.
    pub mass: f64,
    /// Stiffness of the restoring force.
    pub stiffness: f64,
    /// Damping ratio: `< 1` overshoots, `1` settles fastest without
    /// overshoot, `> 1` settles more slowly without overshoot.
    pub damping_ratio: f64,
    /// Maximum distance to the target that still counts as "arrived".
    pub position_epsilon: f64,
    /// Maximum speed that still counts as "arrived".
    pub velocity_epsilon: f64,
}

impl SpringConfig {
    /// Creates a configuration, validating the physical parameters.
    ///
    /// # Errors
    ///
    /// Returns [`SpringError`] when the mass or stiffness is not positive,
    /// the damping ratio is negative, or any value is not finite.
    pub fn new(mass: f64, stiffness: f64, damping_ratio: f64) -> Result<Self, SpringError> {
        if !mass.is_finite() {
            return Err(SpringError::NonFiniteMass);
        }
        if mass <= 0.0 {
            return Err(SpringError::NonPositiveMass);
        }
        if !stiffness.is_finite() {
            return Err(SpringError::NonFiniteStiffness);
        }
        if stiffness <= 0.0 {
            return Err(SpringError::NonPositiveStiffness);
        }
        if !damping_ratio.is_finite() {
            return Err(SpringError::NonFiniteDamping);
        }
        if damping_ratio < 0.0 {
            return Err(SpringError::NegativeDamping);
        }

        Ok(Self {
            mass,
            stiffness,
            damping_ratio,
            position_epsilon: DEFAULT_POSITION_EPSILON,
            velocity_epsilon: DEFAULT_VELOCITY_EPSILON,
        })
    }

    /// Creates a configuration with `mass = 1`, the usual case for UI motion.
    ///
    /// # Errors
    ///
    /// See [`SpringConfig::new`].
    pub fn unit_mass(stiffness: f64, damping_ratio: f64) -> Result<Self, SpringError> {
        Self::new(1.0, stiffness, damping_ratio)
    }

    /// Creates a configuration from a damping coefficient `c` instead of a
    /// damping ratio.
    ///
    /// # Errors
    ///
    /// See [`SpringConfig::new`].
    pub fn from_damping_coefficient(
        mass: f64,
        stiffness: f64,
        damping_coefficient: f64,
    ) -> Result<Self, SpringError> {
        if !damping_coefficient.is_finite() || damping_coefficient < 0.0 {
            return Err(SpringError::NegativeDamping);
        }
        let zeta = damping_coefficient / (2.0 * math::sqrt(stiffness * mass));
        Self::new(mass, stiffness, zeta)
    }

    /// Creates a unit-mass configuration from Apple's *duration* and
    /// *bounce* (SwiftUI `Spring(duration:bounce:)`, Motion's `bounce`).
    ///
    /// * `duration_s` is the **perceptual duration** `d = 2π / ω`, the period
    ///   of the undamped spring. It is not a settle time.
    /// * `bounce` lies in `(-1, 1]`: `0` is critically damped, positive
    ///   values overshoot, negative values are overdamped.
    ///
    /// The mapping is `k = (2π / d)²` and `zeta = 1 - bounce` for
    /// `bounce >= 0`, `zeta = 1 / (1 + bounce)` for `bounce < 0`, so the
    /// damping coefficient is `c = 4π·zeta / d`. (The formula printed in the
    /// WWDC23 transcript, `1 - 4π·bounce / d`, is garbled: at bounce 0 it
    /// would give `zeta ≈ 0.04`.)
    ///
    /// A negative bounce slows the arrival (as in SwiftUI's plain mapping);
    /// unlike Motion, the frequency is not rescaled to compensate.
    ///
    /// ```
    /// use weaver_motion::SpringConfig;
    ///
    /// // Apple's documented example: stiffness 157.9, damping 17.6.
    /// let config = SpringConfig::from_duration_bounce(0.5, 0.3)?;
    /// assert!((config.stiffness - 157.9).abs() < 0.05);
    /// assert!((config.damping_coefficient() - 17.6).abs() < 0.05);
    /// assert!((config.bounce() - 0.3).abs() < 1e-12);
    /// # Ok::<(), weaver_motion::SpringError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`SpringError::InvalidDuration`] unless `duration_s` is finite and
    /// positive, [`SpringError::InvalidBounce`] unless `bounce` is finite and
    /// in `(-1, 1]`.
    pub fn from_duration_bounce(duration_s: f64, bounce: f64) -> Result<Self, SpringError> {
        if !duration_s.is_finite() || duration_s <= 0.0 {
            return Err(SpringError::InvalidDuration);
        }
        if !bounce.is_finite() || bounce <= -1.0 || bounce > 1.0 {
            return Err(SpringError::InvalidBounce);
        }
        let zeta = if bounce >= 0.0 {
            1.0 - bounce
        } else {
            1.0 / (1.0 + bounce)
        };
        Self::from_response(duration_s, zeta)
    }

    /// Creates a unit-mass configuration from a *response* (perceptual
    /// duration, `2π / ω`, in seconds) and a damping ratio.
    ///
    /// This is SwiftUI's `spring(response:dampingFraction:)`: the damping
    /// fraction is the damping ratio.
    ///
    /// # Errors
    ///
    /// [`SpringError::InvalidDuration`] for a non-finite or non-positive
    /// response; otherwise see [`SpringConfig::new`].
    pub fn from_response(response_s: f64, damping_ratio: f64) -> Result<Self, SpringError> {
        if !response_s.is_finite() || response_s <= 0.0 {
            return Err(SpringError::InvalidDuration);
        }
        let omega = 2.0 * core::f64::consts::PI / response_s;
        Self::unit_mass(omega * omega, damping_ratio)
    }

    /// Sets a per-unit settle threshold.
    ///
    /// The position tolerance becomes `threshold` and the velocity tolerance
    /// `threshold * omega`, so a settled spring's remaining motion
    /// (`|v| / omega`, the distance its velocity could still carry it) is
    /// also within `threshold`. Use the unit constants:
    /// [`SETTLE_POSITION_PX`], [`SETTLE_SCALE`], [`SETTLE_OPACITY`],
    /// [`SETTLE_COLOR`].
    ///
    /// ```
    /// use weaver_motion::{SpringConfig, SETTLE_SCALE};
    ///
    /// let scale = SpringConfig::unit_mass(1400.0, 0.9)?.with_settle_threshold(SETTLE_SCALE)?;
    /// assert_eq!(scale.position_epsilon, SETTLE_SCALE);
    /// # Ok::<(), weaver_motion::SpringError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`SpringError`] for non-finite or negative thresholds.
    pub fn with_settle_threshold(self, threshold: f64) -> Result<Self, SpringError> {
        let omega = self.omega();
        self.with_position_epsilon(threshold)?
            .with_velocity_epsilon(threshold * omega)
    }

    /// Overrides the settling tolerance for position.
    ///
    /// # Errors
    ///
    /// Returns [`SpringError`] for non-finite or negative values.
    pub fn with_position_epsilon(mut self, epsilon: f64) -> Result<Self, SpringError> {
        if !epsilon.is_finite() {
            return Err(SpringError::NonFinitePositionEpsilon);
        }
        if epsilon < 0.0 {
            return Err(SpringError::NegativePositionEpsilon);
        }
        self.position_epsilon = epsilon;
        Ok(self)
    }

    /// Overrides the settling tolerance for velocity.
    ///
    /// # Errors
    ///
    /// Returns [`SpringError`] for non-finite or negative values.
    pub fn with_velocity_epsilon(mut self, epsilon: f64) -> Result<Self, SpringError> {
        if !epsilon.is_finite() {
            return Err(SpringError::NonFiniteVelocityEpsilon);
        }
        if epsilon < 0.0 {
            return Err(SpringError::NegativeVelocityEpsilon);
        }
        self.velocity_epsilon = epsilon;
        Ok(self)
    }

    /// Natural angular frequency `omega = sqrt(k / m)`.
    pub fn omega(&self) -> f64 {
        math::sqrt(self.stiffness / self.mass)
    }

    /// Damping coefficient `c = 2 * zeta * sqrt(k * m)`.
    pub fn damping_coefficient(&self) -> f64 {
        2.0 * self.damping_ratio * math::sqrt(self.stiffness * self.mass)
    }

    /// Period of an undamped oscillation, in seconds.
    pub fn natural_period(&self) -> f64 {
        2.0 * core::f64::consts::PI / self.omega()
    }

    /// The damping ratio the solver actually uses: the stored ratio, raised
    /// to [`MIN_DAMPING_RATIO`] if it is lower.
    pub fn effective_damping_ratio(&self) -> f64 {
        if self.damping_ratio < MIN_DAMPING_RATIO {
            MIN_DAMPING_RATIO
        } else {
            self.damping_ratio
        }
    }

    /// Apple's perceptual duration `d = 2π / omega` (`2π / sqrt(k)` at unit
    /// mass), in seconds.
    ///
    /// This is the number to key UI logic (completion callbacks, the next
    /// step of a sequence) off: it is stable across damping ratios. It is
    /// the same value as [`SpringConfig::natural_period`].
    pub fn perceptual_duration(&self) -> f64 {
        self.natural_period()
    }

    /// Apple's bounce: `1 - zeta` for `zeta <= 1`, `1 / zeta - 1` above.
    ///
    /// Bounce is not overshoot: bounce 0.3 overshoots about 4.6%. See
    /// [`SpringConfig::overshoot`].
    pub fn bounce(&self) -> f64 {
        let zeta = self.damping_ratio;
        if zeta <= 1.0 {
            1.0 - zeta
        } else {
            1.0 / zeta - 1.0
        }
    }

    /// Fractional overshoot of a step from rest:
    /// `exp(-π·zeta / sqrt(1 - zeta²))` for `zeta < 1`, else `0`.
    ///
    /// `0.046` means the value passes the target by 4.6% of the distance
    /// travelled. Uses [`SpringConfig::effective_damping_ratio`].
    pub fn overshoot(&self) -> f64 {
        let zeta = self.effective_damping_ratio();
        if zeta >= 1.0 {
            return 0.0;
        }
        math::exp(-core::f64::consts::PI * zeta / math::sqrt((1.0 - zeta) * (1.0 + zeta)))
    }

    /// Time, in seconds, after which a unit step from rest stays within
    /// `epsilon` of the target: the last time `|x(t)| > epsilon` for
    /// `x(0) = 1`, `v(0) = 0`.
    ///
    /// This is Apple's `settlingDuration` (which uses `epsilon = 0.001`). It
    /// measures displacement only and ignores the configured tolerances.
    ///
    /// The result is **computed numerically from the closed form**, not
    /// estimated from the decay envelope: for an underdamped spring the
    /// extrema of a step from rest lie exactly at `t = nπ / omega_d`, so the
    /// last extremum outside the band is found directly and the crossing
    /// after it is located by bisection. Critically and overdamped steps are
    /// monotonic and are bisected directly. The result is accurate to
    /// floating-point resolution.
    ///
    /// Returns `0` for `epsilon >= 1` and infinity for a non-positive or NaN
    /// `epsilon`.
    ///
    /// ```
    /// use weaver_motion::SpringConfig;
    ///
    /// // Critically damped: the exact 2% settle is 5.834 / omega.
    /// let config = SpringConfig::unit_mass(100.0, 1.0)?;
    /// assert!((config.settle_time(0.02) - 0.5834).abs() < 1e-3);
    /// # Ok::<(), weaver_motion::SpringError>(())
    /// ```
    pub fn settle_time(&self, epsilon: f64) -> f64 {
        if epsilon.is_nan() || epsilon <= 0.0 {
            return f64::INFINITY;
        }
        if epsilon >= 1.0 {
            return 0.0;
        }

        let outside = |t: f64| math::abs(solve(self, 1.0, 0.0, 0.0, t).0) > epsilon;
        let zeta = self.effective_damping_ratio();
        let omega = self.omega();

        let (mut lo, mut hi) = if zeta < 1.0 - CRITICAL_TOLERANCE {
            // Extrema of a step from rest: t_n = n·π/omega_d, |x(t_n)| = OS^n.
            let omega_d = omega * math::sqrt((1.0 - zeta) * (1.0 + zeta));
            let half_period = core::f64::consts::PI / omega_d;
            let ratio = self.overshoot();
            // Largest n with OS^n > epsilon, corrected against the evaluated
            // closed form in case of rounding.
            let mut n = math::floor(math::ln(epsilon) / math::ln(ratio)).max(0.0);
            while n > 0.0 && !outside(n * half_period) {
                n -= 1.0;
            }
            while outside((n + 1.0) * half_period) {
                n += 1.0;
            }
            (n * half_period, (n + 1.0) * half_period)
        } else {
            // Monotonic: grow an upper bound until the step is inside.
            let mut hi = 1.0 / omega;
            let mut doublings = 0;
            while outside(hi) {
                hi *= 2.0;
                doublings += 1;
                if doublings > 2048 || !hi.is_finite() {
                    return f64::INFINITY;
                }
            }
            (0.0, hi)
        };

        // `outside` holds on [lo, s) and fails on (s, hi]: bisect for s.
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if mid <= lo || mid >= hi {
                break;
            }
            if outside(mid) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        hi
    }
}

/// Evaluates the closed-form solution of the damped oscillator.
///
/// The returned pair is `(value, velocity)` after `dt` seconds of motion
/// towards `target`, starting from `value` with `velocity`.
///
/// * `dt <= 0` or non-finite `dt` leaves the state untouched.
/// * The result does **not** depend on the number of steps used to reach it:
///   `solve(... 1.0)` equals a hundred `solve(... 0.01)` calls.
///
/// # Examples
///
/// ```
/// use weaver_motion::{solve, SpringConfig};
///
/// let config = SpringConfig::unit_mass(300.0, 0.8).unwrap();
/// let (value, velocity) = solve(&config, 0.0, 0.0, 1.0, 0.01);
/// assert!(value > 0.0 && value < 1.0);
/// assert!(velocity > 0.0);
/// ```
pub fn solve(config: &SpringConfig, value: f64, velocity: f64, target: f64, dt: f64) -> (f64, f64) {
    if !dt.is_finite() || dt <= 0.0 {
        return (value, velocity);
    }
    if !value.is_finite() || !velocity.is_finite() || !target.is_finite() {
        return (value, velocity);
    }

    let omega = config.omega();
    let zeta = config.effective_damping_ratio();
    let y0 = value - target;
    let v0 = velocity;

    let (y, v) = if zeta < 1.0 - CRITICAL_TOLERANCE {
        // Underdamped: oscillates around the target.
        let alpha = zeta * omega;
        // omega_d = omega * sqrt(1 - zeta^2), written to avoid cancellation.
        let omega_d = omega * math::sqrt((1.0 - zeta) * (1.0 + zeta));
        let a = y0;
        let b = (v0 + alpha * y0) / omega_d;
        let decay = math::exp(-alpha * dt);
        let (sin, cos) = (math::sin(omega_d * dt), math::cos(omega_d * dt));
        let y = decay * (a * cos + b * sin);
        let v = decay * ((b * omega_d - alpha * a) * cos + (-a * omega_d - alpha * b) * sin);
        (y, v)
    } else if zeta <= 1.0 + CRITICAL_TOLERANCE {
        // Critically damped: fastest approach without overshoot.
        let b = v0 + omega * y0;
        let decay = math::exp(-omega * dt);
        let y = (y0 + b * dt) * decay;
        let v = (b - omega * (y0 + b * dt)) * decay;
        (y, v)
    } else {
        // Overdamped: two real, negative exponents, so neither term can
        // overflow for any dt. The slow root is written as
        // -omega / (zeta + root) (its product with the fast root is omega²):
        // the textbook -omega * (zeta - root) cancels catastrophically for a
        // large zeta and would freeze the spring.
        let root = math::sqrt((zeta - 1.0) * (zeta + 1.0));
        let r1 = -omega / (zeta + root);
        let r2 = -omega * (zeta + root);
        let c1 = (v0 - r2 * y0) / (r1 - r2);
        let c2 = y0 - c1;
        let e1 = math::exp(r1 * dt);
        let e2 = math::exp(r2 * dt);
        let y = c1 * e1 + c2 * e2;
        let v = r1 * c1 * e1 + r2 * c2 * e2;
        (y, v)
    };

    (target + y, v)
}

/// A scalar value driven by a damped spring.
///
/// The spring owns its position, velocity and destination. Changing the
/// destination never discards the velocity, which is what makes interrupted
/// animations look continuous.
///
/// ```
/// use weaver_motion::{Spring, SpringConfig};
///
/// let config = SpringConfig::unit_mass(300.0, 0.8).unwrap();
/// let mut spring = Spring::new(0.0, 1.0, config);
///
/// spring.advance(1.0 / 60.0);
/// let velocity_before = spring.velocity();
///
/// // Interrupt: the destination changes, the motion continues.
/// spring.set_target(0.0);
/// assert_eq!(spring.velocity(), velocity_before);
/// assert!(spring.value() > 0.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    value: f64,
    velocity: f64,
    target: f64,
    config: SpringConfig,
}

impl Spring {
    /// Creates a spring at `value`, moving towards `target`.
    pub fn new(value: f64, target: f64, config: SpringConfig) -> Self {
        Self {
            value,
            velocity: 0.0,
            target,
            config,
        }
    }

    /// Creates a spring at rest at `value`.
    pub fn resting(value: f64, config: SpringConfig) -> Self {
        Self {
            value,
            velocity: 0.0,
            target: value,
            config,
        }
    }

    /// Creates a spring with an explicit initial velocity.
    pub fn with_velocity(value: f64, velocity: f64, target: f64, config: SpringConfig) -> Self {
        Self {
            value,
            velocity,
            target,
            config,
        }
    }

    /// Current position.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Current velocity, in units per second.
    pub fn velocity(&self) -> f64 {
        self.velocity
    }

    /// Current destination.
    pub fn target(&self) -> f64 {
        self.target
    }

    /// The configuration in use.
    pub fn config(&self) -> SpringConfig {
        self.config
    }

    /// Replaces the configuration, keeping the current state.
    pub fn set_config(&mut self, config: SpringConfig) {
        self.config = config;
    }

    /// Changes the destination while preserving position **and** velocity.
    ///
    /// Non-finite targets are ignored: a broken value must not poison the
    /// animation state.
    pub fn set_target(&mut self, target: f64) {
        if target.is_finite() {
            self.target = target;
        }
    }

    /// Jumps to `value`, cancelling any motion.
    pub fn snap(&mut self, value: f64) {
        if value.is_finite() {
            self.value = value;
            self.target = value;
            self.velocity = 0.0;
        }
    }

    /// Completes the animation immediately.
    pub fn settle(&mut self) {
        self.value = self.target;
        self.velocity = 0.0;
    }

    /// Returns `true` when the spring reached its destination and stopped.
    pub fn is_settled(&self) -> bool {
        math::abs(self.value - self.target) <= self.config.position_epsilon
            && math::abs(self.velocity) <= self.config.velocity_epsilon
    }

    /// Returns `true` while the spring still has motion left.
    pub fn is_active(&self) -> bool {
        !self.is_settled()
    }

    /// Advances the simulation by `dt` seconds.
    ///
    /// Returns `true` when the spring is still moving afterwards. A settled
    /// spring snaps exactly onto its destination and reports `false`, so
    /// schedulers know they can stop requesting frames.
    pub fn advance(&mut self, dt: f64) -> bool {
        if self.is_settled() {
            self.value = self.target;
            self.velocity = 0.0;
            return false;
        }
        if !dt.is_finite() || dt <= 0.0 {
            return true;
        }

        let (value, velocity) = solve(&self.config, self.value, self.velocity, self.target, dt);
        if !value.is_finite() || !velocity.is_finite() {
            // Numerical blow-up: fall back to the destination instead of
            // propagating NaNs into the presentation state.
            self.value = self.target;
            self.velocity = 0.0;
            return false;
        }

        self.value = value;
        self.velocity = velocity;

        if self.is_settled() {
            self.value = self.target;
            self.velocity = 0.0;
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Independent reference implementation: classic RK4 integration of
    /// `x'' = -(k/m) x - (c/m) x'`. Used to validate the closed form.
    fn reference_integrate(
        config: &SpringConfig,
        value: f64,
        velocity: f64,
        target: f64,
        duration: f64,
        steps: usize,
    ) -> (f64, f64) {
        let h = duration / steps as f64;
        let k = config.stiffness / config.mass;
        let c = config.damping_coefficient() / config.mass;
        let (mut x, mut v) = (value - target, velocity);

        let acc = |x: f64, v: f64| -k * x - c * v;

        for _ in 0..steps {
            let (k1x, k1v) = (v, acc(x, v));
            let (k2x, k2v) = (v + 0.5 * h * k1v, acc(x + 0.5 * h * k1x, v + 0.5 * h * k1v));
            let (k3x, k3v) = (v + 0.5 * h * k2v, acc(x + 0.5 * h * k2x, v + 0.5 * h * k2v));
            let (k4x, k4v) = (v + h * k3v, acc(x + h * k3x, v + h * k3v));

            x += h / 6.0 * (k1x + 2.0 * k2x + 2.0 * k3x + k4x);
            v += h / 6.0 * (k1v + 2.0 * k2v + 2.0 * k3v + k4v);
        }

        (x + target, v)
    }

    #[test]
    fn underdamped_matches_numerical_integration() {
        let config = SpringConfig::new(1.0, 100.0, 0.5).unwrap();
        let (value, velocity) = solve(&config, 0.0, 0.0, 1.0, 0.35);
        let (ref_value, ref_velocity) = reference_integrate(&config, 0.0, 0.0, 1.0, 0.35, 20_000);

        assert!((value - ref_value).abs() < 1e-6, "{value} vs {ref_value}");
        assert!(
            (velocity - ref_velocity).abs() < 1e-6,
            "{velocity} vs {ref_velocity}"
        );
    }

    #[test]
    fn critically_damped_matches_numerical_integration() {
        let config = SpringConfig::new(1.0, 100.0, 1.0).unwrap();
        let (value, velocity) = solve(&config, 0.0, 0.0, 1.0, 0.35);
        let (ref_value, ref_velocity) = reference_integrate(&config, 0.0, 0.0, 1.0, 0.35, 20_000);

        assert!((value - ref_value).abs() < 1e-6, "{value} vs {ref_value}");
        assert!(
            (velocity - ref_velocity).abs() < 1e-6,
            "{velocity} vs {ref_velocity}"
        );
    }

    #[test]
    fn overdamped_matches_numerical_integration() {
        let config = SpringConfig::new(1.0, 100.0, 2.0).unwrap();
        let (value, velocity) = solve(&config, 0.0, 0.0, 1.0, 0.35);
        let (ref_value, ref_velocity) = reference_integrate(&config, 0.0, 0.0, 1.0, 0.35, 20_000);

        assert!((value - ref_value).abs() < 1e-6, "{value} vs {ref_value}");
        assert!(
            (velocity - ref_velocity).abs() < 1e-6,
            "{velocity} vs {ref_velocity}"
        );
    }

    #[test]
    fn critically_damped_has_known_closed_form() {
        // omega = 10, y0 = 1, v0 = 0  =>  y(t) = (1 + 10 t) e^(-10 t)
        let config = SpringConfig::unit_mass(100.0, 1.0).unwrap();
        let (value, velocity) = solve(&config, 1.0, 0.0, 0.0, 0.1);

        let expected = 2.0 * (-1.0f64).exp();
        assert!((value - expected).abs() < 1e-12, "{value} vs {expected}");

        // v(t) = (B - omega (y0 + B t)) e^(-omega t), B = v0 + omega y0 = 10
        let expected_v = (10.0 - 10.0 * (1.0 + 10.0 * 0.1)) * (-1.0f64).exp();
        assert!((velocity - expected_v).abs() < 1e-12);
    }

    #[test]
    fn analytic_velocity_matches_finite_difference() {
        let config = SpringConfig::new(1.2, 240.0, 0.65).unwrap();
        let h = 1e-6;
        let (value, velocity) = solve(&config, 0.25, -0.5, 1.0, 0.2);
        let (value_plus, _) = solve(&config, 0.25, -0.5, 1.0, 0.2 + h);
        let (value_minus, _) = solve(&config, 0.25, -0.5, 1.0, 0.2 - h);

        let numeric = (value_plus - value_minus) / (2.0 * h);
        assert!(
            (velocity - numeric).abs() < 1e-4,
            "{velocity} vs {numeric} (value {value})"
        );
    }

    #[test]
    fn result_is_independent_of_step_size() {
        let config = SpringConfig::new(1.0, 300.0, 0.8).unwrap();

        let mut coarse = Spring::new(0.0, 1.0, config);
        coarse.advance(0.5);

        let mut fine = Spring::new(0.0, 1.0, config);
        for _ in 0..500 {
            fine.advance(0.001);
        }

        assert!((coarse.value() - fine.value()).abs() < 1e-9);
        assert!((coarse.velocity() - fine.velocity()).abs() < 1e-9);
    }

    #[test]
    fn retarget_preserves_position_and_velocity() {
        let config = SpringConfig::unit_mass(300.0, 0.8).unwrap();
        let mut spring = Spring::new(0.0, 1.0, config);

        spring.advance(0.1);
        let value = spring.value();
        let velocity = spring.velocity();

        spring.set_target(0.25);

        assert_eq!(spring.value(), value);
        assert_eq!(spring.velocity(), velocity);
        assert_eq!(spring.target(), 0.25);
    }

    #[test]
    fn retarget_does_not_create_a_discontinuity_in_the_trajectory() {
        let config = SpringConfig::unit_mass(300.0, 0.8).unwrap();
        let mut spring = Spring::new(0.0, 1.0, config);

        spring.advance(0.1);
        let before = spring.value();
        spring.set_target(-1.0);
        spring.advance(1e-9);

        assert!(
            (spring.value() - before).abs() < 1e-6,
            "{} -> {}",
            before,
            spring.value()
        );
    }

    #[test]
    fn settles_and_stops_moving() {
        let config = SpringConfig::unit_mass(300.0, 0.8).unwrap();
        let mut spring = Spring::new(0.0, 1.0, config);

        let mut frames = 0;
        while spring.advance(1.0 / 60.0) {
            frames += 1;
            assert!(frames < 10_000, "spring never settled");
        }

        assert!(!spring.is_active());
        assert_eq!(spring.value(), 1.0);
        assert_eq!(spring.velocity(), 0.0);
    }

    #[test]
    fn underdamped_overshoots_and_overdamped_does_not() {
        let under = SpringConfig::unit_mass(300.0, 0.5).unwrap();
        let mut spring = Spring::new(0.0, 1.0, under);
        let mut overshoot = false;
        for _ in 0..600 {
            spring.advance(1.0 / 120.0);
            if spring.value() > 1.0 {
                overshoot = true;
            }
        }
        assert!(overshoot, "underdamped spring should overshoot");

        let over = SpringConfig::unit_mass(300.0, 1.6).unwrap();
        let mut spring = Spring::new(0.0, 1.0, over);
        let mut overshoot = false;
        for _ in 0..600 {
            spring.advance(1.0 / 120.0);
            if spring.value() > 1.0 {
                overshoot = true;
            }
        }
        assert!(!overshoot, "overdamped spring should not overshoot");
    }

    #[test]
    fn invalid_configurations_are_rejected() {
        assert_eq!(
            SpringConfig::new(0.0, 100.0, 1.0).unwrap_err(),
            SpringError::NonPositiveMass
        );
        assert_eq!(
            SpringConfig::new(-1.0, 100.0, 1.0).unwrap_err(),
            SpringError::NonPositiveMass
        );
        assert_eq!(
            SpringConfig::new(f64::NAN, 100.0, 1.0).unwrap_err(),
            SpringError::NonFiniteMass
        );
        assert_eq!(
            SpringConfig::new(1.0, 0.0, 1.0).unwrap_err(),
            SpringError::NonPositiveStiffness
        );
        assert_eq!(
            SpringConfig::new(1.0, 100.0, -0.1).unwrap_err(),
            SpringError::NegativeDamping
        );
        assert_eq!(
            SpringConfig::new(1.0, 100.0, f64::INFINITY).unwrap_err(),
            SpringError::NonFiniteDamping
        );
    }

    #[test]
    fn non_finite_targets_are_ignored() {
        let config = SpringConfig::unit_mass(300.0, 0.8).unwrap();
        let mut spring = Spring::new(0.0, 1.0, config);
        spring.set_target(f64::NAN);
        assert_eq!(spring.target(), 1.0);
        spring.set_target(f64::INFINITY);
        assert_eq!(spring.target(), 1.0);
    }

    #[test]
    fn invalid_time_does_not_change_the_state() {
        let config = SpringConfig::unit_mass(300.0, 0.8).unwrap();
        let mut spring = Spring::new(0.0, 1.0, config);
        spring.advance(0.1);
        let (value, velocity) = (spring.value(), spring.velocity());

        spring.advance(0.0);
        spring.advance(-1.0);
        spring.advance(f64::NAN);

        assert_eq!(spring.value(), value);
        assert_eq!(spring.velocity(), velocity);
    }

    #[test]
    fn one_second_suspension_is_caught_up_analytically() {
        let config = SpringConfig::unit_mass(300.0, 0.8).unwrap();
        let mut suspended = Spring::new(0.0, 1.0, config);
        suspended.advance(1.0);

        let mut stepped = Spring::new(0.0, 1.0, config);
        for _ in 0..60 {
            stepped.advance(1.0 / 60.0);
        }

        assert!((suspended.value() - stepped.value()).abs() < 1e-9);
    }

    #[test]
    fn duration_bounce_matches_apples_worked_example() {
        // Apple `Spring(duration: 0.5, bounce: 0.3)`: stiffness 157.9, damping 17.6.
        let config = SpringConfig::from_duration_bounce(0.5, 0.3).unwrap();
        assert_eq!(config.mass, 1.0);
        assert!(
            (config.stiffness - 157.91).abs() < 0.01,
            "{}",
            config.stiffness
        );
        assert!((config.damping_coefficient() - 17.59).abs() < 0.01);
        assert!((config.damping_ratio - 0.7).abs() < 1e-12);
        assert!((config.perceptual_duration() - 0.5).abs() < 1e-12);
        assert!((config.bounce() - 0.3).abs() < 1e-12);
        // c = 4π(1 - bounce) / d, not the garbled transcript formula.
        let expected_c = 4.0 * core::f64::consts::PI * 0.7 / 0.5;
        assert!((config.damping_coefficient() - expected_c).abs() < 1e-9);
    }

    #[test]
    fn duration_bounce_zero_is_critically_damped() {
        let config = SpringConfig::from_duration_bounce(0.5, 0.0).unwrap();
        assert_eq!(config.damping_ratio, 1.0);
        assert_eq!(config.overshoot(), 0.0);
        assert_eq!(config.bounce(), 0.0);
    }

    #[test]
    fn negative_bounce_is_overdamped_and_round_trips() {
        let config = SpringConfig::from_duration_bounce(0.5, -0.3).unwrap();
        assert!((config.damping_ratio - 1.0 / 0.7).abs() < 1e-12);
        assert!((config.bounce() + 0.3).abs() < 1e-12);
        for bounce in [-0.9, -0.5, -0.1, 0.0, 0.15, 0.3, 0.7, 1.0] {
            let config = SpringConfig::from_duration_bounce(0.35, bounce).unwrap();
            assert!((config.bounce() - bounce).abs() < 1e-12, "{bounce}");
            assert!((config.perceptual_duration() - 0.35).abs() < 1e-12);
        }
    }

    #[test]
    fn duration_bounce_rejects_invalid_input() {
        for d in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                SpringConfig::from_duration_bounce(d, 0.0).unwrap_err(),
                SpringError::InvalidDuration
            );
        }
        for b in [-1.0, -2.0, 1.01, f64::NAN] {
            assert_eq!(
                SpringConfig::from_duration_bounce(0.5, b).unwrap_err(),
                SpringError::InvalidBounce
            );
        }
        assert_eq!(
            SpringConfig::from_response(0.0, 0.8).unwrap_err(),
            SpringError::InvalidDuration
        );
    }

    #[test]
    fn response_is_swiftui_response_damping_fraction() {
        // SwiftUI `spring()` default: response 0.5, dampingFraction 0.825.
        let config = SpringConfig::from_response(0.5, 0.825).unwrap();
        assert!((config.stiffness - 157.91).abs() < 0.01);
        assert!((config.damping_coefficient() - 20.73).abs() < 0.01);
    }

    #[test]
    fn m3_perceptual_values() {
        let fast = SpringConfig::unit_mass(1400.0, 0.9).unwrap();
        assert!((fast.perceptual_duration() * 1000.0 - 167.9).abs() < 0.1);
        assert!((fast.overshoot() - 0.00152).abs() < 0.00001);
        assert!((fast.bounce() - 0.1).abs() < 1e-12);
        let expressive = SpringConfig::unit_mass(800.0, 0.6).unwrap();
        assert!((expressive.overshoot() - 0.0948).abs() < 0.0001);
        assert!((SpringConfig::unit_mass(380.0, 0.8).unwrap().overshoot() - 0.0152).abs() < 0.0001);
        // Apple: bounce 0.15 overshoots ~0.6%, bounce 0.3 ~4.6%.
        let b15 = SpringConfig::from_duration_bounce(0.5, 0.15).unwrap();
        assert!((b15.overshoot() - 0.006).abs() < 0.0005);
        let b30 = SpringConfig::from_duration_bounce(0.5, 0.3).unwrap();
        assert!((b30.overshoot() - 0.046).abs() < 0.0005);
    }

    #[test]
    fn overshoot_formula_matches_the_trajectory() {
        for zeta in [0.3, 0.6, 0.8, 0.9] {
            let config = SpringConfig::unit_mass(400.0, zeta).unwrap();
            let peak_time = core::f64::consts::PI / (20.0 * (1.0 - zeta * zeta).sqrt());
            let (peak, velocity) = solve(&config, 0.0, 0.0, 1.0, peak_time);
            assert!(velocity.abs() < 1e-9, "{zeta}");
            assert!((peak - 1.0 - config.overshoot()).abs() < 1e-12, "{zeta}");
        }
    }

    /// Brute-force settle time: last sample with |x| > eps on a fine grid.
    fn brute_settle(config: &SpringConfig, eps: f64) -> f64 {
        let h = 1e-5;
        let mut last = 0.0;
        let mut t = 0.0;
        while t < 5.0 {
            if solve(config, 1.0, 0.0, 0.0, t).0.abs() > eps {
                last = t;
            }
            t += h;
        }
        last
    }

    #[test]
    fn settle_time_matches_the_research_tables() {
        let ms = |c: SpringConfig, eps: f64| c.settle_time(eps) * 1000.0;
        let cases = [
            (SpringConfig::unit_mass(1500.0, 1.0).unwrap(), 151.0),
            (SpringConfig::unit_mass(1400.0, 0.9).unwrap(), 126.0),
            (SpringConfig::unit_mass(700.0, 0.9).unwrap(), 178.0),
            (SpringConfig::unit_mass(800.0, 0.6).unwrap(), 210.0),
            (SpringConfig::from_duration_bounce(0.5, 0.0).unwrap(), 464.0),
            (
                SpringConfig::from_duration_bounce(0.5, 0.15).unwrap(),
                333.0,
            ),
            (SpringConfig::from_duration_bounce(0.5, 0.3).unwrap(), 476.0),
            (
                SpringConfig::from_duration_bounce(0.5, -0.3).unwrap(),
                798.0,
            ),
            (
                SpringConfig::unit_mass(280.0, 60.0 / (2.0 * 280f64.sqrt())).unwrap(),
                786.0,
            ),
        ];
        for (config, expected) in cases {
            let got = ms(config, 0.02);
            assert!(
                (got - expected).abs() < 1.0,
                "{config:?}: {got} vs {expected}"
            );
        }
        // Critically damped: 5.834 / omega exactly.
        let crit = SpringConfig::unit_mass(100.0, 1.0).unwrap();
        assert!((crit.settle_time(0.02) - 0.58339).abs() < 1e-4);
    }

    #[test]
    fn settle_time_agrees_with_brute_force() {
        for (k, zeta) in [
            (300.0, 0.3),
            (700.0, 0.9),
            (380.0, 0.8),
            (200.0, 1.0),
            (150.0, 2.5),
        ] {
            let config = SpringConfig::unit_mass(k, zeta).unwrap();
            for eps in [0.02, 1e-3, 1.0 / 255.0] {
                let exact = config.settle_time(eps);
                let brute = brute_settle(&config, eps);
                assert!(
                    (exact - brute).abs() < 2e-5,
                    "k {k} zeta {zeta} eps {eps}: {exact} vs {brute}"
                );
            }
        }
    }

    #[test]
    fn settle_time_edge_cases() {
        let config = SpringConfig::unit_mass(300.0, 0.8).unwrap();
        assert_eq!(config.settle_time(1.0), 0.0);
        assert_eq!(config.settle_time(0.0), f64::INFINITY);
        assert_eq!(config.settle_time(f64::NAN), f64::INFINITY);
        // Tighter bands take longer.
        assert!(config.settle_time(1e-4) > config.settle_time(1e-2));
    }

    #[test]
    fn near_critical_band_is_continuous_and_accurate() {
        // Either side of the critical band and inside it, the closed form
        // stays within the band's O(tolerance) error of a precise reference.
        for zeta in [
            1.0 - 2.0 * CRITICAL_TOLERANCE,
            1.0 - 1.0001 * CRITICAL_TOLERANCE,
            1.0 - 0.9999 * CRITICAL_TOLERANCE,
            1.0 - 1e-9,
            1.0,
            1.0 + 1e-9,
            1.0 + 0.9999 * CRITICAL_TOLERANCE,
            1.0 + 1.0001 * CRITICAL_TOLERANCE,
            1.0 + 2.0 * CRITICAL_TOLERANCE,
        ] {
            let config = SpringConfig::unit_mass(700.0, zeta).unwrap();
            for t in [0.01, 0.1, 0.3] {
                let (x, v) = solve(&config, 0.0, 2.0, 1.0, t);
                let (rx, rv) = reference_integrate(&config, 0.0, 2.0, 1.0, t, 20_000);
                assert!(x.is_finite() && v.is_finite());
                assert!((x - rx).abs() < 1e-4, "zeta {zeta} t {t}: {x} vs {rx}");
                assert!((v - rv).abs() < 1e-2, "zeta {zeta} t {t}: {v} vs {rv}");
            }
        }
    }

    #[test]
    fn heavily_overdamped_spring_is_accurate_and_does_not_freeze() {
        // Slow mode decays at r1 = -omega / (zeta + sqrt(zeta² - 1)). The
        // textbook -omega (zeta - root) cancels to ~4% error at zeta = 1e7
        // and to exactly 0 (a frozen spring) at zeta = 1e8.
        for zeta in [1e7, 1e8] {
            let config = SpringConfig::unit_mass(100.0, zeta).unwrap();
            let r1 = -10.0 / (zeta + (zeta * zeta - 1.0).sqrt());
            let t = -1.0 / r1; // one slow time constant
            let (y, _) = solve(&config, 1.0, 0.0, 0.0, t);
            assert!((y - (-1.0f64).exp()).abs() < 1e-6, "zeta {zeta}: {y}");
        }
    }

    #[test]
    fn overdamped_terms_do_not_overflow_for_large_dt() {
        for zeta in [1.5, 10.0, 1e4] {
            let config = SpringConfig::unit_mass(3800.0, zeta).unwrap();
            for dt in [1.0, 1e3, 1e9, 1e300] {
                let (x, v) = solve(&config, 0.0, 5000.0, 1.0, dt);
                assert!(x.is_finite() && v.is_finite(), "zeta {zeta} dt {dt}");
            }
        }
        let config = SpringConfig::unit_mass(3800.0, 3.0).unwrap();
        let (x, v) = solve(&config, 0.0, 5000.0, 1.0, 1e9);
        assert_eq!((x, v), (1.0, 0.0));
    }

    #[test]
    fn damping_ratio_has_a_floor() {
        let undamped = SpringConfig::unit_mass(300.0, 0.0).unwrap();
        assert_eq!(undamped.damping_ratio, 0.0);
        assert_eq!(undamped.effective_damping_ratio(), MIN_DAMPING_RATIO);
        let floor = SpringConfig::unit_mass(300.0, MIN_DAMPING_RATIO).unwrap();
        assert_eq!(
            solve(&undamped, 0.0, 0.0, 1.0, 0.7),
            solve(&floor, 0.0, 0.0, 1.0, 0.7)
        );
        // Without the floor this spring would oscillate forever.
        let mut spring = Spring::new(0.0, 1.0, undamped);
        let mut frames = 0;
        while spring.advance(1.0 / 60.0) {
            frames += 1;
            assert!(frames < 60 * 60, "an undamped spring must still settle");
        }
    }

    #[test]
    fn settle_threshold_scales_velocity_tolerance_with_omega() {
        let config = SpringConfig::unit_mass(1600.0, 1.0)
            .unwrap()
            .with_settle_threshold(SETTLE_OPACITY)
            .unwrap();
        assert_eq!(config.position_epsilon, SETTLE_OPACITY);
        assert!((config.velocity_epsilon - 40.0 / 255.0).abs() < 1e-12);
        assert!(SpringConfig::unit_mass(1.0, 1.0)
            .unwrap()
            .with_settle_threshold(-1.0)
            .is_err());
    }

    #[test]
    fn no_time_error_accumulates_over_many_frames() {
        // Ten seconds of 144 Hz frames against one exact evaluation.
        let config = SpringConfig::unit_mass(30.0, 0.15)
            .unwrap()
            .with_position_epsilon(0.0)
            .unwrap()
            .with_velocity_epsilon(0.0)
            .unwrap();
        let mut stepped = Spring::new(0.0, 1.0, config);
        for _ in 0..1440 {
            stepped.advance(1.0 / 144.0);
        }
        let (value, velocity) = solve(&config, 0.0, 0.0, 1.0, 10.0);
        assert!((stepped.value() - value).abs() < 1e-12);
        assert!((stepped.velocity() - velocity).abs() < 1e-10);
    }
}
