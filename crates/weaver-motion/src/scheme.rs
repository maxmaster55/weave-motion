//! Motion schemes: a complete set of springs, one per motion role.
//!
//! A [`MotionScheme`] answers "which spring does this kind of movement use?"
//! for a whole interface at once, so swapping the scheme changes the feel of
//! everything consistently. Roles split into two families:
//!
//! * **spatial** springs move things: position, size, shape, scale. In the
//!   expressive scheme they overshoot a little.
//! * **effects** springs change how things look: opacity and colour. They are
//!   critically damped in every scheme, because a colour that overshoots
//!   looks like a glitch rather than a bounce.
//!
//! Each family has a fast, default and slow speed: fast for small parts and
//! direct feedback (a pressed button), default for everyday movement, slow for
//! large surfaces and full-screen transitions.
//!
//! The `standard` and `expressive` values follow the shape of Material 3
//! Expressive's published motion schemes; `calm` is Weave's own, with no
//! overshoot at all. Like [`MotionToken`](crate::MotionToken), they are
//! starting points to tune per product, not official tokens.
//!
//! | Scheme | Spatial fast | Spatial default | Spatial slow | Effects fast / default / slow |
//! |---|---|---|---|---|
//! | `standard` | 1400, 0.9 | 700, 0.9 | 300, 0.9 | 3800 / 1600 / 800, 1.0 |
//! | `expressive` | 800, 0.6 | 380, 0.8 | 200, 0.8 | 3800 / 1600 / 800, 1.0 |
//! | `calm` | 1000, 1.0 | 500, 1.0 | 240, 1.0 | 3800 / 1600 / 800, 1.0 |
//!
//! Values are stiffness (unit mass) and damping ratio.
//!
//! ```
//! use weaver_motion::{Animated, MotionRole, MotionScheme};
//!
//! let scheme = MotionScheme::expressive();
//! let mut scale = Animated::new(1.0_f64, 0.96, scheme.spring(MotionRole::SpatialFast));
//! scale.advance(1.0 / 60.0);
//! assert!(scale.value() < 1.0);
//! ```

use crate::spring::SpringConfig;

/// The kind of movement a spring is used for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionRole {
    /// Small parts and direct feedback: presses, toggles, small shape changes.
    SpatialFast,
    /// Everyday movement: cards, tiles, menus, layout changes.
    SpatialDefault,
    /// Large surfaces: panels and full-screen transitions.
    SpatialSlow,
    /// Quick opacity and colour changes on small parts.
    EffectsFast,
    /// Everyday opacity and colour changes.
    EffectsDefault,
    /// Opacity and colour changes on large surfaces.
    EffectsSlow,
}

impl MotionRole {
    /// Every role, in a stable order.
    pub const ALL: [MotionRole; 6] = [
        MotionRole::SpatialFast,
        MotionRole::SpatialDefault,
        MotionRole::SpatialSlow,
        MotionRole::EffectsFast,
        MotionRole::EffectsDefault,
        MotionRole::EffectsSlow,
    ];

    /// The role's snake_case name.
    pub fn name(self) -> &'static str {
        match self {
            MotionRole::SpatialFast => "spatial_fast",
            MotionRole::SpatialDefault => "spatial_default",
            MotionRole::SpatialSlow => "spatial_slow",
            MotionRole::EffectsFast => "effects_fast",
            MotionRole::EffectsDefault => "effects_default",
            MotionRole::EffectsSlow => "effects_slow",
        }
    }

    /// Whether the role moves things (as opposed to changing how they look).
    pub fn is_spatial(self) -> bool {
        matches!(
            self,
            MotionRole::SpatialFast | MotionRole::SpatialDefault | MotionRole::SpatialSlow
        )
    }
}

/// One spring per [`MotionRole`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionScheme {
    /// Spring for [`MotionRole::SpatialFast`].
    pub spatial_fast: SpringConfig,
    /// Spring for [`MotionRole::SpatialDefault`].
    pub spatial_default: SpringConfig,
    /// Spring for [`MotionRole::SpatialSlow`].
    pub spatial_slow: SpringConfig,
    /// Spring for [`MotionRole::EffectsFast`].
    pub effects_fast: SpringConfig,
    /// Spring for [`MotionRole::EffectsDefault`].
    pub effects_default: SpringConfig,
    /// Spring for [`MotionRole::EffectsSlow`].
    pub effects_slow: SpringConfig,
}

fn spring(stiffness: f64, damping_ratio: f64) -> SpringConfig {
    SpringConfig::unit_mass(stiffness, damping_ratio).expect("valid scheme spring")
}

impl MotionScheme {
    /// Brisk, barely-overshooting motion for everyday tools.
    pub fn standard() -> Self {
        Self::with_spatial([(1400.0, 0.9), (700.0, 0.9), (300.0, 0.9)])
    }

    /// Springier motion with a visible bounce on spatial movement.
    pub fn expressive() -> Self {
        Self::with_spatial([(800.0, 0.6), (380.0, 0.8), (200.0, 0.8)])
    }

    /// No overshoot anywhere: critically damped spatial springs.
    pub fn calm() -> Self {
        Self::with_spatial([(1000.0, 1.0), (500.0, 1.0), (240.0, 1.0)])
    }

    fn with_spatial(spatial: [(f64, f64); 3]) -> Self {
        Self {
            spatial_fast: spring(spatial[0].0, spatial[0].1),
            spatial_default: spring(spatial[1].0, spatial[1].1),
            spatial_slow: spring(spatial[2].0, spatial[2].1),
            effects_fast: spring(3800.0, 1.0),
            effects_default: spring(1600.0, 1.0),
            effects_slow: spring(800.0, 1.0),
        }
    }

    /// The spring this scheme uses for `role`.
    pub fn spring(&self, role: MotionRole) -> SpringConfig {
        match role {
            MotionRole::SpatialFast => self.spatial_fast,
            MotionRole::SpatialDefault => self.spatial_default,
            MotionRole::SpatialSlow => self.spatial_slow,
            MotionRole::EffectsFast => self.effects_fast,
            MotionRole::EffectsDefault => self.effects_default,
            MotionRole::EffectsSlow => self.effects_slow,
        }
    }

    /// This scheme with the spring for `role` replaced, for tuning.
    pub fn with(mut self, role: MotionRole, config: SpringConfig) -> Self {
        match role {
            MotionRole::SpatialFast => self.spatial_fast = config,
            MotionRole::SpatialDefault => self.spatial_default = config,
            MotionRole::SpatialSlow => self.spatial_slow = config,
            MotionRole::EffectsFast => self.effects_fast = config,
            MotionRole::EffectsDefault => self.effects_default = config,
            MotionRole::EffectsSlow => self.effects_slow = config,
        }
        self
    }
}

impl Default for MotionScheme {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schemes() -> [MotionScheme; 3] {
        [
            MotionScheme::standard(),
            MotionScheme::expressive(),
            MotionScheme::calm(),
        ]
    }

    #[test]
    fn effects_never_overshoot() {
        for scheme in schemes() {
            for role in MotionRole::ALL.into_iter().filter(|r| !r.is_spatial()) {
                assert!(scheme.spring(role).damping_ratio >= 1.0, "{}", role.name());
            }
        }
    }

    #[test]
    fn faster_roles_are_stiffer() {
        for scheme in schemes() {
            assert!(scheme.spatial_fast.stiffness > scheme.spatial_default.stiffness);
            assert!(scheme.spatial_default.stiffness > scheme.spatial_slow.stiffness);
            assert!(scheme.effects_fast.stiffness > scheme.effects_default.stiffness);
            assert!(scheme.effects_default.stiffness > scheme.effects_slow.stiffness);
        }
    }

    #[test]
    fn expressive_bounces_more_than_standard_and_calm_not_at_all() {
        let (standard, expressive, calm) = (
            MotionScheme::standard(),
            MotionScheme::expressive(),
            MotionScheme::calm(),
        );
        for role in MotionRole::ALL.into_iter().filter(|r| r.is_spatial()) {
            assert!(expressive.spring(role).damping_ratio <= standard.spring(role).damping_ratio);
            assert!(calm.spring(role).damping_ratio >= 1.0);
        }
        assert!(expressive.spatial_fast.damping_ratio < standard.spatial_fast.damping_ratio);
    }

    #[test]
    fn with_replaces_only_one_role() {
        let custom = SpringConfig::unit_mass(123.0, 0.5).unwrap();
        let scheme = MotionScheme::expressive().with(MotionRole::SpatialSlow, custom);
        for role in MotionRole::ALL {
            let expected = if role == MotionRole::SpatialSlow {
                custom
            } else {
                MotionScheme::expressive().spring(role)
            };
            assert_eq!(scheme.spring(role), expected, "{}", role.name());
        }
    }

    #[test]
    fn role_names_are_stable() {
        let names: [&str; 6] = MotionRole::ALL.map(MotionRole::name);
        assert_eq!(
            names,
            [
                "spatial_fast",
                "spatial_default",
                "spatial_slow",
                "effects_fast",
                "effects_default",
                "effects_slow"
            ]
        );
    }
}
