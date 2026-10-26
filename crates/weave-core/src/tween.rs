//! Tweens and easing curves.
//!
//! Springs are the default for spatial motion. Some effects — opacity,
//! colour, blur — read better with a fixed duration: they have no physical
//! meaning, so a spring would only add overshoot. Weave therefore ships
//! tweens as a separate primitive instead of forcing everything through the
//! same model.

use crate::animatable::Animatable;

/// An easing curve.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Easing {
    /// Constant speed.
    #[default]
    Linear,
    /// Accelerates from the start.
    EaseIn,
    /// Decelerates towards the end.
    EaseOut,
    /// Accelerates, then decelerates.
    EaseInOut,
    /// A CSS-style cubic bezier, `(x1, y1, x2, y2)`.
    CubicBezier(f64, f64, f64, f64),
}

impl Easing {
    /// Maps a linear progress `t` in `[0, 1]` to an eased progress.
    ///
    /// ```
    /// use weave_core::Easing;
    ///
    /// assert_eq!(Easing::Linear.apply(0.25), 0.25);
    /// assert_eq!(Easing::EaseInOut.apply(0.0), 0.0);
    /// assert_eq!(Easing::EaseInOut.apply(1.0), 1.0);
    /// ```
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match *self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    let mirrored = 1.0 - t;
                    1.0 - 2.0 * mirrored * mirrored
                }
            }
            Easing::CubicBezier(x1, y1, x2, y2) => cubic_bezier_ease(t, x1, y1, x2, y2),
        }
    }
}

/// Evaluates a CSS-style cubic bezier easing curve.
///
/// The bezier is parameterised by `u`, with `x(u)` mapping to the input
/// progress and `y(u)` to the output progress. `u` is found with Newton
/// iterations and falls back to bisection when the curve is flat.
fn cubic_bezier_ease(t: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }

    let sample = |u: f64| -> (f64, f64) {
        // Polynomial bezier with P0 = (0, 0) and P3 = (1, 1).
        let cx = 3.0 * x1;
        let bx = 3.0 * (x2 - x1) - cx;
        let ax = 1.0 - cx - bx;
        let cy = 3.0 * y1;
        let by = 3.0 * (y2 - y1) - cy;
        let ay = 1.0 - cy - by;
        (((ax * u + bx) * u + cx) * u, ((ay * u + by) * u + cy) * u)
    };

    let slope = |u: f64| -> f64 {
        let cx = 3.0 * x1;
        let bx = 3.0 * (x2 - x1) - cx;
        let ax = 1.0 - cx - bx;
        3.0 * ax * u * u + 2.0 * bx * u + cx
    };

    // Newton-Raphson.
    let mut u = t;
    for _ in 0..8 {
        let (x, _) = sample(u);
        let dx = slope(u);
        if dx.abs() < 1e-6 {
            break;
        }
        u -= (x - t) / dx;
        if !(0.0..=1.0).contains(&u) {
            u = t;
            break;
        }
    }

    // Bisection fallback when Newton did not converge.
    if (sample(u).0 - t).abs() > 1e-6 {
        let (mut low, mut high) = (0.0_f64, 1.0_f64);
        u = t;
        for _ in 0..32 {
            let (x, _) = sample(u);
            if (x - t).abs() < 1e-6 {
                break;
            }
            if x < t {
                low = u;
            } else {
                high = u;
            }
            u = 0.5 * (low + high);
        }
    }

    sample(u).1
}

/// Timing configuration of a tween.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TweenConfig {
    /// Total duration in seconds, excluding the delay.
    pub duration: f64,
    /// Delay before the tween starts, in seconds.
    pub delay: f64,
    /// The easing curve.
    pub easing: Easing,
}

impl TweenConfig {
    /// Creates a linear tween of `duration` seconds.
    pub fn new(duration: f64) -> Self {
        Self {
            duration: duration.max(0.0),
            delay: 0.0,
            easing: Easing::Linear,
        }
    }

    /// Sets the easing curve.
    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the delay in seconds.
    pub fn with_delay(mut self, delay: f64) -> Self {
        self.delay = delay.max(0.0);
        self
    }
}

impl Default for TweenConfig {
    fn default() -> Self {
        Self::new(0.2)
    }
}

/// A value that interpolates towards a destination over a fixed duration.
///
/// Tweens have no physical model, so they carry no velocity. They are meant
/// for effects (opacity, colour) rather than for spatial motion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tween<T: Animatable> {
    from: T,
    to: T,
    value: T,
    elapsed: f64,
    config: TweenConfig,
}

impl<T: Animatable> Tween<T> {
    /// Creates a tween from `from` to `to`.
    pub fn new(from: T, to: T, config: TweenConfig) -> Self {
        Self {
            from,
            to,
            value: from,
            elapsed: 0.0,
            config,
        }
    }

    /// Current value.
    pub fn value(&self) -> T {
        self.value
    }

    /// The destination.
    pub fn target(&self) -> T {
        self.to
    }

    /// The timing configuration.
    pub fn config(&self) -> TweenConfig {
        self.config
    }

    /// Linear progress in `[0, 1]`, before easing.
    pub fn progress(&self) -> f64 {
        if self.config.duration <= 0.0 {
            return 1.0;
        }
        let active = (self.elapsed - self.config.delay).max(0.0);
        (active / self.config.duration).clamp(0.0, 1.0)
    }

    /// Eased progress in `[0, 1]`.
    pub fn eased_progress(&self) -> f64 {
        self.config.easing.apply(self.progress())
    }

    /// Creates a tween that is already finished, holding `value`.
    pub fn resting(value: T, config: TweenConfig) -> Self {
        Self {
            from: value,
            to: value,
            value,
            elapsed: config.delay + config.duration,
            config,
        }
    }

    /// Changes the destination, restarting from the current value.
    ///
    /// Restarting from the current value avoids a visible jump, which is the
    /// tween counterpart of the spring's velocity preservation.
    pub fn set_target(&mut self, to: T) {
        self.from = self.value;
        self.to = to;
        self.elapsed = 0.0;
        self.value = self.from;
    }

    /// Jumps to `value` and cancels the tween.
    pub fn snap(&mut self, value: T) {
        self.from = value;
        self.to = value;
        self.value = value;
        self.elapsed = 0.0;
    }

    /// Returns `true` when the tween reached its destination.
    pub fn is_settled(&self) -> bool {
        self.progress() >= 1.0
    }

    /// Returns `true` while the tween is still running.
    pub fn is_active(&self) -> bool {
        !self.is_settled()
    }

    /// Advances the tween by `dt` seconds.
    ///
    /// Returns `true` while the tween is still running.
    pub fn advance(&mut self, dt: f64) -> bool {
        if !dt.is_finite() || dt <= 0.0 {
            return self.is_active();
        }

        self.elapsed += dt;
        let progress = self.progress();
        self.value = T::lerp(self.from, self.to, self.config.easing.apply(progress));

        if progress >= 1.0 {
            self.value = self.to;
            self.elapsed = self.config.delay + self.config.duration;
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_easing_is_identity() {
        for t in [0.0, 0.1, 0.5, 0.9, 1.0] {
            assert!((Easing::Linear.apply(t) - t).abs() < 1e-12);
        }
    }

    #[test]
    fn easings_are_monotonic_and_bounded() {
        let easings = [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::CubicBezier(0.4, 0.0, 0.2, 1.0),
        ];
        for easing in easings {
            let mut previous = -1.0;
            for i in 0..=100 {
                let t = i as f64 / 100.0;
                let value = easing.apply(t);
                assert!((0.0..=1.0).contains(&value), "{easing:?} produced {value}");
                assert!(value >= previous, "{easing:?} is not monotonic");
                previous = value;
            }
            assert!(easing.apply(0.0).abs() < 1e-9);
            assert!((easing.apply(1.0) - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn tween_reaches_its_destination() {
        let mut tween = Tween::new(0.0_f64, 1.0, TweenConfig::new(0.2));
        assert!(tween.is_active());

        let mut frames = 0;
        while tween.advance(1.0 / 60.0) {
            frames += 1;
            assert!(frames < 1000);
        }

        assert_eq!(tween.value(), 1.0);
        assert!(!tween.is_active());
    }

    #[test]
    fn retargeting_a_tween_restarts_from_the_current_value() {
        let mut tween = Tween::new(0.0_f64, 1.0, TweenConfig::new(1.0));
        tween.advance(0.25);
        let value = tween.value();
        assert!(value > 0.0);

        tween.set_target(0.0);
        assert_eq!(tween.value(), value);
        assert_eq!(tween.progress(), 0.0);
    }

    #[test]
    fn tween_delay_is_respected() {
        let mut tween = Tween::new(0.0_f64, 1.0, TweenConfig::new(0.1).with_delay(0.05));
        tween.advance(0.04);
        assert_eq!(tween.value(), 0.0);
        tween.advance(0.02);
        assert!(tween.value() > 0.0);
    }
}
