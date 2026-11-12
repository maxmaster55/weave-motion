//! Motion tokens: Weave's original named presets, **superseded by
//! [`MotionScheme`] roles**.
//!
//! These presets predate the role scheme. They are kept, with unchanged
//! values, so existing code (weave-scene, the examples) keeps compiling and
//! behaving the same. New code should pick a [`MotionRole`] from a
//! [`MotionScheme`] instead; `docs/MOTION.md` explains how.
//!
//! Measured against the motion research behind `docs/MOTION.md`, the old
//! spatial presets run long: their perceptual durations are 281–531 ms and
//! their 1e-3 settle times 391–780 ms, past the ~500 ms UI ceiling, and
//! `expressive_spatial` overshoots 6.8%. They also duplicate the scheme
//! roles. [`MotionToken::role`] gives each token's replacement:
//!
//! | Token | Stiffness | Damping ratio | Perceptual duration | Replacement role |
//! |---|---:|---:|---:|---|
//! | `fast_spatial` | 500 | 0.85 | 281 ms | standard `SpatialDefault` |
//! | `default_spatial` | 300 | 0.80 | 363 ms | standard `SpatialSlow` |
//! | `expressive_spatial` | 220 | 0.65 | 424 ms | **expressive** `SpatialDefault` |
//! | `gentle_spatial` | 140 | 1.00 | 531 ms | none for movement; only non-interactive background dimming, as `EffectsSlow` |
//! | `effects` | tween 150 ms | – | – | `EffectsDefault` |
//!
//! [`MotionToken::replacement_spring`] returns the replacement role's spring
//! from the right scheme.

use crate::scheme::{MotionRole, MotionScheme};
use crate::spring::SpringConfig;
use crate::tween::{Easing, TweenConfig};

/// A named motion preset, superseded by [`MotionRole`].
///
/// The values are unchanged and kept for existing code. Mapping to roles
/// (see `docs/MOTION.md`): `FastSpatial` → standard `SpatialDefault`,
/// `DefaultSpatial` → standard `SpatialSlow`, `ExpressiveSpatial` →
/// expressive `SpatialDefault`, `GentleSpatial` → only non-interactive
/// background dimming (`EffectsSlow`), `Effects` → `EffectsDefault`.
/// [`MotionToken::role`] and [`MotionToken::replacement_spring`] return them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionToken {
    /// Fast, barely-overshooting feedback for direct manipulation.
    FastSpatial,
    /// The everyday default for spatial movement.
    DefaultSpatial,
    /// Softer, more visible bounce for emphasised transitions.
    ExpressiveSpatial,
    /// Critically damped motion for large surfaces.
    GentleSpatial,
    /// Short tween for opacity and colour effects.
    Effects,
}

/// Alias kept for the API sketched in `examples/API_SKETCH.rs`.
pub type MotionPreset = MotionToken;

impl MotionToken {
    /// Every token, in a stable order.
    pub const ALL: [MotionToken; 5] = [
        MotionToken::FastSpatial,
        MotionToken::DefaultSpatial,
        MotionToken::ExpressiveSpatial,
        MotionToken::GentleSpatial,
        MotionToken::Effects,
    ];

    /// The token's snake_case name.
    pub fn name(self) -> &'static str {
        match self {
            MotionToken::FastSpatial => "fast_spatial",
            MotionToken::DefaultSpatial => "default_spatial",
            MotionToken::ExpressiveSpatial => "expressive_spatial",
            MotionToken::GentleSpatial => "gentle_spatial",
            MotionToken::Effects => "effects",
        }
    }

    /// The spring configuration described by the token.
    ///
    /// The [`MotionToken::Effects`] token has no spring flavour and returns
    /// `None`; use [`MotionToken::tween_config`] for it.
    pub fn spring_config(self) -> Option<SpringConfig> {
        let (stiffness, damping_ratio) = match self {
            MotionToken::FastSpatial => (500.0, 0.85),
            MotionToken::DefaultSpatial => (300.0, 0.80),
            MotionToken::ExpressiveSpatial => (220.0, 0.65),
            MotionToken::GentleSpatial => (140.0, 1.00),
            MotionToken::Effects => return None,
        };
        Some(SpringConfig::unit_mass(stiffness, damping_ratio).expect("valid token"))
    }

    /// The [`MotionRole`] that replaces this token.
    ///
    /// `ExpressiveSpatial` maps to `SpatialDefault` *of the expressive
    /// scheme*; [`MotionToken::replacement_spring`] accounts for that.
    /// `GentleSpatial` has no spatial replacement: its 780 ms settle is too
    /// slow for anything interactive, and its only remaining use is
    /// non-interactive background dimming, an `EffectsSlow` job.
    pub const fn role(self) -> MotionRole {
        match self {
            MotionToken::FastSpatial => MotionRole::SpatialDefault,
            MotionToken::DefaultSpatial => MotionRole::SpatialSlow,
            MotionToken::ExpressiveSpatial => MotionRole::SpatialDefault,
            MotionToken::GentleSpatial => MotionRole::EffectsSlow,
            MotionToken::Effects => MotionRole::EffectsDefault,
        }
    }

    /// The spring of this token's replacement role: from
    /// [`MotionScheme::expressive`] for `ExpressiveSpatial`, from
    /// [`MotionScheme::standard`] otherwise.
    pub fn replacement_spring(self) -> SpringConfig {
        let scheme = match self {
            MotionToken::ExpressiveSpatial => MotionScheme::expressive(),
            _ => MotionScheme::standard(),
        };
        scheme.spring(self.role())
    }

    /// The tween configuration described by the token.
    ///
    /// Spatial tokens have no tween flavour and return `None`.
    pub fn tween_config(self) -> Option<TweenConfig> {
        match self {
            MotionToken::Effects => Some(TweenConfig::new(0.15).with_easing(Easing::Linear)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spatial_tokens_expose_a_spring() {
        for token in MotionToken::ALL {
            if token == MotionToken::Effects {
                assert!(token.spring_config().is_none());
            } else {
                let config = token.spring_config().expect("spatial token");
                assert!(config.mass > 0.0);
                assert!(config.stiffness > 0.0);
                assert!(config.damping_ratio >= 0.0);
            }
        }
    }

    #[test]
    fn effects_token_exposes_a_tween() {
        let config = MotionToken::Effects.tween_config().expect("effects token");
        assert!((config.duration - 0.15).abs() < 1e-12);
        assert!(MotionToken::DefaultSpatial.tween_config().is_none());
    }

    #[test]
    fn token_names_are_stable() {
        assert_eq!(MotionToken::FastSpatial.name(), "fast_spatial");
        assert_eq!(MotionToken::DefaultSpatial.name(), "default_spatial");
        assert_eq!(MotionToken::ExpressiveSpatial.name(), "expressive_spatial");
        assert_eq!(MotionToken::GentleSpatial.name(), "gentle_spatial");
        assert_eq!(MotionToken::Effects.name(), "effects");
    }

    #[test]
    fn spring_tokens_are_ordered_by_speed() {
        let fast = MotionToken::FastSpatial.spring_config().unwrap();
        let default = MotionToken::DefaultSpatial.spring_config().unwrap();
        let expressive = MotionToken::ExpressiveSpatial.spring_config().unwrap();
        let gentle = MotionToken::GentleSpatial.spring_config().unwrap();

        assert!(fast.omega() > default.omega());
        assert!(default.omega() > expressive.omega());
        assert!(expressive.omega() > gentle.omega());
    }

    #[test]
    fn tokens_map_to_the_report_roles() {
        assert_eq!(MotionToken::FastSpatial.role(), MotionRole::SpatialDefault);
        assert_eq!(MotionToken::DefaultSpatial.role(), MotionRole::SpatialSlow);
        assert_eq!(
            MotionToken::ExpressiveSpatial.role(),
            MotionRole::SpatialDefault
        );
        assert_eq!(MotionToken::GentleSpatial.role(), MotionRole::EffectsSlow);
        assert_eq!(MotionToken::Effects.role(), MotionRole::EffectsDefault);
        assert_eq!(
            MotionToken::ExpressiveSpatial.replacement_spring(),
            MotionScheme::expressive().spatial_default
        );
        assert_eq!(
            MotionToken::DefaultSpatial.replacement_spring(),
            MotionScheme::standard().spatial_slow
        );
    }

    #[test]
    fn replacements_are_no_slower_than_the_old_tokens() {
        for token in MotionToken::ALL {
            let Some(old) = token.spring_config() else {
                continue;
            };
            let new = token.replacement_spring();
            assert!(
                new.settle_time(1e-3) < old.settle_time(1e-3),
                "{}",
                token.name()
            );
            assert!(new.overshoot() <= old.overshoot(), "{}", token.name());
        }
    }

    #[test]
    fn documented_perceptual_durations() {
        let ms =
            |t: MotionToken| (t.spring_config().unwrap().perceptual_duration() * 1000.0).round();
        assert_eq!(ms(MotionToken::FastSpatial), 281.0);
        assert_eq!(ms(MotionToken::DefaultSpatial), 363.0);
        assert_eq!(ms(MotionToken::ExpressiveSpatial), 424.0);
        assert_eq!(ms(MotionToken::GentleSpatial), 531.0);
    }
}
