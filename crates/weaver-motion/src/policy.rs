//! Accessibility policy: how much motion an application is allowed to show.
//!
//! Reduced motion is not "no motion": it **removes or replaces** the motion
//! that can make people uncomfortable while keeping the information a
//! transition carries (what appeared, what moved, what is focused). Snapping
//! everything loses object constancy and, in Apple's words, can make an app
//! "more confusing".
//!
//! ## Role-aware mapping
//!
//! [`MotionPolicy::spring_for_role`] implements the mapping in
//! `docs/MOTION.md`:
//!
//! | Policy | Spatial roles | Effects roles |
//! |---|---|---|
//! | `Full` | unchanged | unchanged |
//! | `Reduced` | same stiffness tier, critically damped: no bounce, still moving | unchanged (opacity and colour are not "motion" under WCAG 2.3.3) |
//! | `Off` | snap | snap |
//!
//! The older [`MotionPolicy::spring_config`] treats every spring alike and is
//! kept unchanged for callers without a role.
//!
//! ## Substitution belongs to the content layer
//!
//! Some reduced-motion decisions depend on what is moving, which this crate
//! cannot know: large travel becomes a crossfade in place, ambient loops
//! stop and show a static state, an entrance cascade becomes one group
//! fade. Those choices belong to the layer that owns the content (for
//! example a tile layer's motion code). This module gives it the inputs:
//! [`CROSSFADE_DISTANCE_PX`] and [`MotionPolicy::prefers_crossfade`].

use crate::scheme::{MotionRole, MotionScheme};
use crate::spring::SpringConfig;
use crate::tween::{Easing, TweenConfig};

/// Travel distance, in logical pixels, above which reduced motion should
/// replace a move with a crossfade in place (or an instant move plus a short
/// highlight fade).
///
/// About one tile width. Large travel relative to the viewport is a known
/// vestibular trigger, but no source gives a numeric limit: this value is
/// an inference (tagged I in `docs/MOTION.md`), to be tuned per product.
pub const CROSSFADE_DISTANCE_PX: f64 = 96.0;

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

    /// Adapts the scheme's spring for `role` to the policy.
    ///
    /// * [`MotionPolicy::Full`]: the scheme's spring, unchanged.
    /// * [`MotionPolicy::Reduced`]: spatial roles keep their stiffness tier
    ///   but become critically damped (a damping ratio below 1 is raised to
    ///   1; overdamped springs are left alone), so they still move but never
    ///   bounce. Effects roles are unchanged.
    /// * [`MotionPolicy::Off`]: every role snaps. The returned spring is the
    ///   very stiff, critically damped one [`MotionPolicy::spring_config`]
    ///   returns for `Off` (settled within about 50 ms); callers that drive
    ///   values directly should snap them instead, as
    ///   [`MotionEngine::retarget`](crate::MotionEngine::retarget) does.
    ///
    /// Whether a large move should become a crossfade instead is the
    /// caller's decision; see [`MotionPolicy::prefers_crossfade`].
    ///
    /// ```
    /// use weaver_motion::{MotionPolicy, MotionRole, MotionScheme};
    ///
    /// let scheme = MotionScheme::expressive();
    /// let fast = MotionPolicy::Reduced.spring_for_role(&scheme, MotionRole::SpatialFast);
    /// assert_eq!(fast.damping_ratio, 1.0);
    /// assert_eq!(fast.stiffness, scheme.spatial_fast.stiffness);
    ///
    /// let fade = MotionPolicy::Reduced.spring_for_role(&scheme, MotionRole::EffectsDefault);
    /// assert_eq!(fade, scheme.effects_default);
    /// ```
    pub fn spring_for_role(&self, scheme: &MotionScheme, role: MotionRole) -> SpringConfig {
        self.spring_config_for_role(role, scheme.spring(role))
    }

    /// Adapts an arbitrary spring used for `role` to the policy, with the
    /// same mapping as [`MotionPolicy::spring_for_role`]. Use it for tuned
    /// springs that are not part of a scheme.
    pub fn spring_config_for_role(&self, role: MotionRole, base: SpringConfig) -> SpringConfig {
        match self {
            MotionPolicy::Full => base,
            MotionPolicy::Reduced if role.is_spatial() => {
                let mut adjusted = base;
                if adjusted.damping_ratio < 1.0 {
                    adjusted.damping_ratio = 1.0;
                }
                adjusted
            }
            MotionPolicy::Reduced => base,
            MotionPolicy::Off => MotionPolicy::Off.spring_config(base),
        }
    }

    /// `true` when a move of `distance_px` logical pixels should be replaced
    /// by a crossfade in place (or an instant move plus a highlight fade)
    /// instead of travelling.
    ///
    /// Only [`MotionPolicy::Reduced`] substitutes, and only for travel longer
    /// than [`CROSSFADE_DISTANCE_PX`]. Under [`MotionPolicy::Off`] everything
    /// snaps, so there is nothing to crossfade; under
    /// [`MotionPolicy::Full`] the move animates. Zooms to full window and
    /// re-sorts should be treated as large travel regardless of distance.
    ///
    /// ```
    /// use weaver_motion::MotionPolicy;
    ///
    /// assert!(MotionPolicy::Reduced.prefers_crossfade(400.0));
    /// assert!(!MotionPolicy::Reduced.prefers_crossfade(8.0));
    /// assert!(!MotionPolicy::Full.prefers_crossfade(400.0));
    /// ```
    pub fn prefers_crossfade(&self, distance_px: f64) -> bool {
        matches!(self, MotionPolicy::Reduced)
            && crate::math::abs(distance_px) > CROSSFADE_DISTANCE_PX
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
    fn full_role_policy_keeps_the_scheme() {
        for scheme in [MotionScheme::standard(), MotionScheme::expressive()] {
            for role in MotionRole::ALL {
                assert_eq!(
                    MotionPolicy::Full.spring_for_role(&scheme, role),
                    scheme.spring(role)
                );
            }
        }
    }

    #[test]
    fn reduced_role_policy_removes_bounce_but_keeps_the_tier() {
        let scheme = MotionScheme::expressive();
        for role in MotionRole::ALL {
            let base = scheme.spring(role);
            let reduced = MotionPolicy::Reduced.spring_for_role(&scheme, role);
            assert_eq!(reduced.stiffness, base.stiffness, "{}", role.name());
            assert_eq!(reduced.position_epsilon, base.position_epsilon);
            if role.is_spatial() {
                assert_eq!(reduced.damping_ratio, 1.0, "{}", role.name());
                assert_eq!(reduced.overshoot(), 0.0);
                // Still moving: same perceptual duration, not a snap.
                assert_eq!(reduced.perceptual_duration(), base.perceptual_duration());
            } else {
                assert_eq!(reduced, base, "{}", role.name());
            }
        }
    }

    #[test]
    fn reduced_role_policy_keeps_overdamped_springs() {
        let soft = SpringConfig::unit_mass(200.0, 1.5).unwrap();
        let reduced = MotionPolicy::Reduced.spring_config_for_role(MotionRole::SpatialSlow, soft);
        assert_eq!(reduced, soft);
    }

    #[test]
    fn off_role_policy_snaps_everything() {
        let scheme = MotionScheme::standard();
        for role in MotionRole::ALL {
            let off = MotionPolicy::Off.spring_for_role(&scheme, role);
            let mut spring = crate::Spring::new(0.0, 1.0, off);
            spring.advance(0.05);
            assert!((spring.value() - 1.0).abs() < 0.02, "{}", role.name());
        }
    }

    #[test]
    fn crossfade_only_under_reduced_and_for_large_travel() {
        assert!(MotionPolicy::Reduced.prefers_crossfade(CROSSFADE_DISTANCE_PX + 1.0));
        assert!(MotionPolicy::Reduced.prefers_crossfade(-500.0));
        assert!(!MotionPolicy::Reduced.prefers_crossfade(CROSSFADE_DISTANCE_PX));
        assert!(!MotionPolicy::Full.prefers_crossfade(1000.0));
        assert!(!MotionPolicy::Off.prefers_crossfade(1000.0));
    }

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
