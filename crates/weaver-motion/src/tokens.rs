//! Motion tokens: named starting points for spring and tween behaviour.
//!
//! These presets are **Weave's own**, not official Material tokens. They are
//! starting points that still need perceptual tuning per product: a spring
//! that feels right for a 40 px button overshoots badly on a 900 px panel.
//!
//! | Token | Stiffness | Damping ratio | Typical use |
//! |---|---:|---:|---|
//! | `fast_spatial` | 500 | 0.85 | press / hover / focus feedback |
//! | `default_spatial` | 300 | 0.80 | menus, cards, everyday movement |
//! | `expressive_spatial` | 220 | 0.65 | hero transitions with a soft bounce |
//! | `gentle_spatial` | 140 | 1.00 | large surfaces, restrained motion |
//! | `effects` | tween | – | opacity and colour, ~150 ms |

use crate::spring::SpringConfig;
use crate::tween::{Easing, TweenConfig};

/// A named motion preset.
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
}
