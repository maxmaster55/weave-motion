//! Accessibility policy: how much motion an application is allowed to show.
//!
//! Reduced motion is not "no motion": it is "no *displacement*". Weave keeps
//! the information a transition carries (what appeared, what moved, what is
//! focused) while removing the movement that can make people uncomfortable.

use crate::spring::SpringConfig;
use crate::tween::{Easing, TweenConfig};

/// How much motion the engine is allowed to produce.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MotionPolicy {
    /// Full motion: springs behave exactly as configured.
    #[default]
    Full,
    /// Reduced motion: springs lose their overshoot and get faster, tweens
    /// become short and linear. Position changes still happen, but they are
    /// quick and never bouncy.
    Reduced,
    /// No motion at all: values jump to their destination.
    Off,
}

impl MotionPolicy {
    /// Adapts a spring configuration to the policy.
    ///
    /// * [`MotionPolicy::Full`] keeps the configuration untouched.
    /// * [`MotionPolicy::Reduced`] makes the spring critically damped and
    ///   stiffer, so it settles quickly without overshooting.
    /// * [`MotionPolicy::Off`] makes the spring effectively instantaneous.
    pub fn spring_config(&self, base: SpringConfig) -> SpringConfig {
        match self {
            MotionPolicy::Full => base,
            MotionPolicy::Reduced => {
                let mut adjusted = base;
                adjusted.damping_ratio = 1.0;
                adjusted.stiffness = base.stiffness * 2.0;
                adjusted
            }
            MotionPolicy::Off => {
                let mut adjusted = base;
                // A very stiff, critically damped spring is indistinguishable
                // from a snap for all practical purposes.
                adjusted.damping_ratio = 1.0;
                adjusted.stiffness = base.stiffness * 100.0;
                adjusted
            }
        }
    }

    /// Adapts a tween configuration to the policy.
    pub fn tween_config(&self, base: TweenConfig) -> TweenConfig {
        match self {
            MotionPolicy::Full => base,
            MotionPolicy::Reduced => TweenConfig {
                duration: base.duration.min(0.12),
                delay: base.delay.min(0.05),
                easing: Easing::Linear,
            },
            MotionPolicy::Off => TweenConfig {
                duration: 0.0,
                delay: 0.0,
                easing: Easing::Linear,
            },
        }
    }

    /// A scale factor applied to durations.
    pub fn duration_scale(&self) -> f64 {
        match self {
            MotionPolicy::Full => 1.0,
            MotionPolicy::Reduced => 0.5,
            MotionPolicy::Off => 0.0,
        }
    }

    /// `true` when the policy allows any displacement at all.
    pub fn allows_motion(&self) -> bool {
        !matches!(self, MotionPolicy::Off)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_policy_keeps_configurations() {
        let spring = SpringConfig::unit_mass(300.0, 0.8).unwrap();
        let tween = TweenConfig::new(0.2);
        assert_eq!(MotionPolicy::Full.spring_config(spring), spring);
        assert_eq!(MotionPolicy::Full.tween_config(tween), tween);
    }

    #[test]
    fn reduced_policy_removes_overshoot() {
        let spring = SpringConfig::unit_mass(300.0, 0.5).unwrap();
        let adjusted = MotionPolicy::Reduced.spring_config(spring);
        assert_eq!(adjusted.damping_ratio, 1.0);
        assert!(adjusted.stiffness > spring.stiffness);

        // A critically damped spring never overshoots.
        use crate::spring::solve;
        let mut value = 0.0;
        let mut velocity = 0.0;
        for _ in 0..600 {
            let (v, vel) = solve(&adjusted, value, velocity, 1.0, 1.0 / 120.0);
            value = v;
            velocity = vel;
            assert!(value <= 1.0);
        }
    }

    #[test]
    fn reduced_policy_shortens_tweens() {
        let tween = TweenConfig::new(1.0);
        let adjusted = MotionPolicy::Reduced.tween_config(tween);
        assert!(adjusted.duration < tween.duration);
        assert_eq!(adjusted.easing, Easing::Linear);
    }

    #[test]
    fn off_policy_makes_motion_immediate() {
        let tween = TweenConfig::new(0.2);
        assert_eq!(MotionPolicy::Off.tween_config(tween).duration, 0.0);
        assert!(!MotionPolicy::Off.allows_motion());
    }
}
