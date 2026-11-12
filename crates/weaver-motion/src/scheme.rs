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
//! The `standard` and `expressive` values match Material 3's published
//! spring tokens (`StandardMotionTokens.kt`, `ExpressiveMotionTokens.kt`);
//! `calm` is Weave's own, with no overshoot at all. They are starting points
//! to tune per product, not official tokens.
//!
//! **Standard is the default** ([`MotionScheme::default`]) and the right
//! choice for always-visible, productive surfaces: shells, dashboards,
//! widgets. **Expressive is opt-in** per surface, for rare user-initiated
//! moments such as completion or onboarding: its fast spatial spring
//! overshoots about 9.5%, and the rest of it is mainly slower. See
//! `docs/MOTION.md` for how to choose a role.
//!
//! | Scheme | Spatial fast | Spatial default | Spatial slow | Effects fast / default / slow |
//! |---|---|---|---|---|
//! | `standard` | 1400, 0.9 | 700, 0.9 | 300, 0.9 | 3800 / 1600 / 800, 1.0 |
//! | `expressive` | 800, 0.6 | 380, 0.8 | 200, 0.8 | 3800 / 1600 / 800, 1.0 |
//! | `calm` | 1000, 1.0 | 500, 1.0 | 240, 1.0 | 3800 / 1600 / 800, 1.0 |
//!
//! Values are stiffness (unit mass) and damping ratio.
//!
//! | Role | Perceptual duration `2π/√k` (standard · expressive) | Overshoot (standard · expressive) |
//! |---|---|---|
//! | spatial fast | 168 ms · 222 ms | 0.15% · 9.5% |
//! | spatial default | 237 ms · 322 ms | 0.15% · 1.5% |
//! | spatial slow | 363 ms · 444 ms | 0.15% · 1.5% |
//! | effects fast / default / slow | 102 / 157 / 222 ms | none |
//!
//! ## Settle thresholds
//!
//! Effects roles always animate opacity or a normalised colour channel, so
//! they settle at [`EFFECTS_EPSILON`] (one 8-bit step, 1/255) via
//! [`SpringConfig::with_settle_threshold`]: the final snap is invisible and
//! the spring stops sooner than with a tighter tolerance. Spatial roles do
//! not imply a unit (they drive pixels *and* scale factors), so they keep the
//! default tolerance (0.01 units, well under half a pixel). For scale, apply
//! [`SETTLE_SCALE`](crate::SETTLE_SCALE); for pixels,
//! [`SETTLE_POSITION_PX`](crate::SETTLE_POSITION_PX) may be used to stop
//! sooner:
//!
//! ```
//! use weaver_motion::{MotionRole, MotionScheme, SETTLE_SCALE};
//!
//! let press = MotionScheme::default()
//!     .spring(MotionRole::SpatialFast)
//!     .with_settle_threshold(SETTLE_SCALE)?;
//! # let _ = press;
//! # Ok::<(), weaver_motion::SpringError>(())
//! ```
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

/// Settle threshold of every effects spring: one 8-bit channel step
/// (1/255, [`SETTLE_OPACITY`](crate::SETTLE_OPACITY)).
///
/// It is the position tolerance; the velocity tolerance is
/// `EFFECTS_EPSILON * omega` (see [`SpringConfig::with_settle_threshold`]).
/// Before the per-unit thresholds this was `1e-3` for both, which kept
/// effects springs running 30–60% past the point where any change is
/// visible.
pub const EFFECTS_EPSILON: f64 = 1.0 / 255.0;

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
    pub const fn name(self) -> &'static str {
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
    pub const fn is_spatial(self) -> bool {
        matches!(
            self,
            MotionRole::SpatialFast | MotionRole::SpatialDefault | MotionRole::SpatialSlow
        )
    }

    /// Whether the role changes how things look (opacity, colour).
    pub const fn is_effects(self) -> bool {
        !self.is_spatial()
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

fn effects(stiffness: f64) -> SpringConfig {
    spring(stiffness, 1.0)
        .with_settle_threshold(EFFECTS_EPSILON)
        .expect("valid effects tolerance")
}

impl MotionScheme {
    /// Brisk, barely-overshooting motion for everyday tools (Material 3
    /// standard). The default, and the right scheme for always-visible
    /// surfaces such as shells, dashboards and widgets.
    pub fn standard() -> Self {
        Self::with_spatial([(1400.0, 0.9), (700.0, 0.9), (300.0, 0.9)])
    }

    /// Springier motion with a visible bounce on spatial movement (Material
    /// 3 expressive). Opt-in per surface for rare, user-initiated moments;
    /// not for always-visible productive surfaces.
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
            effects_fast: effects(3800.0),
            effects_default: effects(1600.0),
            effects_slow: effects(800.0),
        }
    }

    /// The spring this scheme uses for `role`.
    pub const fn spring(&self, role: MotionRole) -> SpringConfig {
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
    /// [`MotionScheme::standard`].
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
    fn default_is_standard() {
        assert_eq!(MotionScheme::default(), MotionScheme::standard());
    }

    #[test]
    fn roles_split_into_spatial_and_effects() {
        let spatial: usize = MotionRole::ALL.iter().filter(|r| r.is_spatial()).count();
        assert_eq!(spatial, 3);
        for role in MotionRole::ALL {
            assert_ne!(role.is_spatial(), role.is_effects(), "{}", role.name());
            assert_eq!(role.is_spatial(), role.name().starts_with("spatial"));
        }
    }

    #[test]
    fn effects_settle_within_the_tight_tolerance() {
        for scheme in schemes() {
            for role in MotionRole::ALL.into_iter().filter(|r| r.is_effects()) {
                let spring = scheme.spring(role);
                assert_eq!(spring.position_epsilon, EFFECTS_EPSILON);
                assert_eq!(spring.velocity_epsilon, EFFECTS_EPSILON * spring.omega());
            }
        }
    }

    /// Steps a 0 -> 1 spring until it settles; returns (peak value, steps).
    fn trajectory(config: SpringConfig) -> (f64, usize) {
        let mut s = crate::Spring::new(0.0, 1.0, config);
        let mut peak = 0.0_f64;
        for step in 1..=4800 {
            let active = s.advance(1.0 / 240.0);
            peak = peak.max(s.value());
            if !active {
                return (peak, step);
            }
        }
        (peak, usize::MAX)
    }

    #[test]
    fn trajectories_overshoot_only_where_intended_and_always_settle() {
        for (name, scheme) in ["standard", "expressive", "calm"].iter().zip(schemes()) {
            for role in MotionRole::ALL {
                let (peak, steps) = trajectory(scheme.spring(role));
                assert!(steps <= 4800, "{name} {} never settled", role.name());
                let overshoots = peak > 1.0 + 1e-6;
                let should = role.is_spatial() && *name != "calm";
                assert_eq!(overshoots, should, "{name} {} peak {peak}", role.name());
            }
        }
    }

    #[test]
    fn engine_uses_the_active_scheme_for_roles() {
        let run = |scheme: MotionScheme| {
            let mut engine = crate::MotionEngine::new();
            engine.set_scheme(scheme);
            assert_eq!(engine.scheme(), scheme);
            let id = engine.spring_for_role("x", 0.0, MotionRole::SpatialFast);
            engine.tick(0.0);
            engine.retarget(id, 1.0);
            let mut expected = crate::Spring::new(0.0, 1.0, scheme.spatial_fast);
            let mut peak = 0.0_f64;
            for frame in 1..=240 {
                engine.tick(frame as f64 / 120.0);
                expected.advance(1.0 / 120.0);
                let value = engine.value_f64(id).unwrap();
                assert!((value - expected.value()).abs() < 1e-9, "frame {frame}");
                peak = peak.max(value);
            }
            peak
        };
        assert!(run(MotionScheme::expressive()) > 1.0);
        assert!(run(MotionScheme::calm()) <= 1.0 + 1e-9);
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

    #[test]
    fn perceptual_values_match_the_m3_tables() {
        let ms = |c: SpringConfig| (c.perceptual_duration() * 1000.0).round();
        let (standard, expressive) = (MotionScheme::standard(), MotionScheme::expressive());
        assert_eq!(ms(standard.spatial_fast), 168.0);
        assert_eq!(ms(standard.spatial_default), 237.0);
        assert_eq!(ms(standard.spatial_slow), 363.0);
        assert_eq!(ms(expressive.spatial_fast), 222.0);
        assert_eq!(ms(expressive.spatial_default), 322.0);
        assert_eq!(ms(expressive.spatial_slow), 444.0);
        assert_eq!(ms(standard.effects_fast), 102.0);
        assert_eq!(ms(standard.effects_default), 157.0);
        assert_eq!(ms(standard.effects_slow), 222.0);

        let pct = |c: SpringConfig| (c.overshoot() * 1000.0).round() / 10.0;
        assert_eq!(pct(standard.spatial_fast), 0.2); // 0.15%
        assert_eq!(pct(expressive.spatial_fast), 9.5);
        assert_eq!(pct(expressive.spatial_default), 1.5);
        assert_eq!(standard.effects_default.overshoot(), 0.0);
    }

    #[test]
    fn effects_stop_sooner_with_the_per_unit_threshold() {
        // Frames until an effects spring reports settled, at 240 Hz.
        let frames = |config: SpringConfig| trajectory(config).1;
        for stiffness in [3800.0, 1600.0, 800.0] {
            let old = SpringConfig::unit_mass(stiffness, 1.0)
                .unwrap()
                .with_position_epsilon(1e-3)
                .unwrap()
                .with_velocity_epsilon(1e-3)
                .unwrap();
            let new = effects(stiffness);
            assert!(frames(new) < frames(old), "k {stiffness}");
        }
    }
}
