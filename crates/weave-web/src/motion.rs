//! Handle-based springs for JavaScript.
//!
//! A [`Motion`] owns a set of springs addressed by small integer handles, so
//! only numbers cross the wasm boundary. JavaScript asks for values and
//! applies them; Weave decides how they move.
//!
//! The same API is implemented in plain JavaScript by `js/weave.js`, so a web
//! application can choose the wasm backend or the JS backend (see
//! `Weave.createMotion({ backend })`) without changing its code.
//!
//! The type is plain Rust; with the `web` feature it is also exported through
//! `wasm-bindgen` with camelCase method names.

use alloc::vec::Vec;
use weaver_motion::{MotionPolicy, MotionRole, MotionScheme, MotionToken, Spring, SpringConfig};

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

/// Maps a preset name to its legacy token. Unknown names fall back to the default.
///
/// Accepts both the short (`"fast"`) and the token (`"fast_spatial"`) names.
pub fn preset(name: &str) -> MotionToken {
    match name {
        "fast" | "fast_spatial" => MotionToken::FastSpatial,
        "expressive" | "expressive_spatial" => MotionToken::ExpressiveSpatial,
        "gentle" | "gentle_spatial" => MotionToken::GentleSpatial,
        _ => MotionToken::DefaultSpatial,
    }
}

/// Maps a role or legacy preset name to an M3 [`MotionRole`].
///
/// Accepts role names (`"spatial_fast"`, `"spatial_default"`, `"spatial_slow"`,
/// `"effects_fast"`, `"effects_default"`, `"effects_slow"`) as well as legacy
/// preset names (`"fast"`, `"default"`, `"expressive"`, `"gentle"`, `"effects"`).
pub fn role(name: &str) -> MotionRole {
    match name {
        "spatial_fast" | "fast" | "fast_spatial" => MotionRole::SpatialFast,
        "spatial_slow" | "slow" | "gentle" | "gentle_spatial" => MotionRole::SpatialSlow,
        "effects_fast" => MotionRole::EffectsFast,
        "effects_default" | "effects" => MotionRole::EffectsDefault,
        "effects_slow" => MotionRole::EffectsSlow,
        _ => MotionRole::SpatialDefault,
    }
}

fn scheme_by_name(name: &str) -> MotionScheme {
    match name {
        "expressive" => MotionScheme::expressive(),
        "calm" => MotionScheme::calm(),
        _ => MotionScheme::standard(),
    }
}

/// A set of springs advanced together by one frame clock.
#[cfg_attr(feature = "web", wasm_bindgen)]
#[derive(Debug, Default)]
pub struct Motion {
    springs: Vec<Option<(Spring, SpringConfig, MotionRole)>>,
    free: Vec<u32>,
    policy: MotionPolicy,
    scheme: MotionScheme,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl Motion {
    /// Creates an empty set with the default (standard M3) motion scheme.
    /// `reduced` follows the user's reduced-motion preference: no overshoot
    /// and a faster settle ([`MotionPolicy::Reduced`]).
    #[cfg_attr(feature = "web", wasm_bindgen(constructor))]
    pub fn new(reduced: bool) -> Motion {
        Motion::with_scheme(MotionScheme::default(), reduced)
    }

    /// Creates an empty set configured with the M3 expressive motion scheme.
    #[cfg_attr(feature = "web", wasm_bindgen(js_name = newExpressive))]
    pub fn new_expressive(reduced: bool) -> Motion {
        Motion::with_scheme(MotionScheme::expressive(), reduced)
    }

    /// Switches between full and reduced motion. Springs keep their state.
    #[cfg_attr(feature = "web", wasm_bindgen(js_name = setReduced))]
    pub fn set_reduced(&mut self, reduced: bool) {
        self.policy = if reduced {
            MotionPolicy::Reduced
        } else {
            MotionPolicy::Full
        };
        for (spring, base, role) in self.springs.iter_mut().flatten() {
            spring.set_config(self.policy.spring_config_for_role(*role, *base));
        }
    }

    /// Switches the active M3 scheme by name (`"standard"`, `"expressive"`, or `"calm"`).
    /// Existing springs keep their current position and velocity.
    #[cfg_attr(feature = "web", wasm_bindgen(js_name = setScheme))]
    pub fn set_scheme_name(&mut self, name: &str) {
        self.set_scheme(scheme_by_name(name));
    }

    /// Creates a spring resting at `value` and returns its handle.
    ///
    /// `preset`: an M3 role (`"spatial_fast"`, `"spatial_default"`,
    /// `"spatial_slow"`, `"effects_fast"`, `"effects_default"`, `"effects_slow"`)
    /// or a legacy preset (`"fast"`, `"default"`, `"expressive"`, `"gentle"`).
    pub fn spring(&mut self, value: f64, preset: &str) -> u32 {
        let r = role(preset);
        let scheme = match preset {
            "expressive" | "expressive_spatial" => MotionScheme::expressive(),
            _ => self.scheme,
        };
        let base = scheme.spring(r);
        self.alloc_spring(value, base, r)
    }

    /// Frees a handle; it may be reused by a later [`Motion::spring`].
    pub fn release(&mut self, id: u32) {
        if let Some(slot) = self.springs.get_mut(id as usize) {
            if slot.take().is_some() {
                self.free.push(id);
            }
        }
    }

    /// Changes the destination. Position and velocity carry through.
    #[cfg_attr(feature = "web", wasm_bindgen(js_name = setTarget))]
    pub fn set_target(&mut self, id: u32, target: f64) {
        if let Some(Some((s, _, _))) = self.springs.get_mut(id as usize) {
            s.set_target(target);
        }
    }

    /// Jumps to `value` and stops.
    pub fn snap(&mut self, id: u32, value: f64) {
        if let Some(Some((s, _, _))) = self.springs.get_mut(id as usize) {
            s.snap(value);
        }
    }

    /// Current presented value (0 for an unknown handle).
    pub fn value(&self, id: u32) -> f64 {
        match self.springs.get(id as usize) {
            Some(Some((s, _, _))) => s.value(),
            _ => 0.0,
        }
    }

    /// Current velocity (0 for an unknown handle).
    pub fn velocity(&self, id: u32) -> f64 {
        match self.springs.get(id as usize) {
            Some(Some((s, _, _))) => s.velocity(),
            _ => 0.0,
        }
    }

    /// Destination (0 for an unknown handle).
    pub fn target(&self, id: u32) -> f64 {
        match self.springs.get(id as usize) {
            Some(Some((s, _, _))) => s.target(),
            _ => 0.0,
        }
    }

    /// Whether the spring still has motion left.
    #[cfg_attr(feature = "web", wasm_bindgen(js_name = isActive))]
    pub fn is_active(&self, id: u32) -> bool {
        matches!(self.springs.get(id as usize), Some(Some((s, _, _))) if s.is_active())
    }

    /// Advances every spring by `dt` seconds (clamped to 0.1 s so a long frame
    /// gap cannot jump an animation). Returns `true` while anything moves.
    pub fn advance(&mut self, dt: f64) -> bool {
        let dt = dt.clamp(0.0, 0.1);
        let mut active = false;
        for (s, _, _) in self.springs.iter_mut().flatten() {
            active |= s.advance(dt);
        }
        active
    }
}

impl Motion {
    /// Creates an empty set with the given [`MotionScheme`] and reduced-motion flag.
    pub fn with_scheme(scheme: MotionScheme, reduced: bool) -> Motion {
        Motion {
            springs: Vec::new(),
            free: Vec::new(),
            policy: if reduced {
                MotionPolicy::Reduced
            } else {
                MotionPolicy::Full
            },
            scheme,
        }
    }

    /// Switches the active [`MotionScheme`]. Existing springs keep their state.
    pub fn set_scheme(&mut self, scheme: MotionScheme) {
        self.scheme = scheme;
        for (spring, base, role) in self.springs.iter_mut().flatten() {
            *base = scheme.spring(*role);
            spring.set_config(self.policy.spring_config_for_role(*role, *base));
        }
    }

    /// Creates a spring resting at `value` for the given M3 [`MotionRole`].
    pub fn spring_role(&mut self, value: f64, role: MotionRole) -> u32 {
        let base = self.scheme.spring(role);
        self.alloc_spring(value, base, role)
    }

    fn alloc_spring(&mut self, value: f64, base: SpringConfig, role: MotionRole) -> u32 {
        let cfg = self.policy.spring_config_for_role(role, base);
        let entry = Some((Spring::resting(value, cfg), base, role));
        if let Some(id) = self.free.pop() {
            self.springs[id as usize] = entry;
            id
        } else {
            self.springs.push(entry);
            (self.springs.len() - 1) as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(m: &mut Motion) {
        for _ in 0..1200 {
            if !m.advance(1.0 / 60.0) {
                break;
            }
        }
    }

    #[test]
    fn reaches_target_and_settles() {
        let mut m = Motion::new(false);
        let id = m.spring(0.0, "default");
        m.set_target(id, 100.0);
        run(&mut m);
        assert!(!m.is_active(id));
        assert_eq!(m.value(id), 100.0);
    }

    #[test]
    fn interruption_keeps_velocity() {
        let mut m = Motion::new(false);
        let id = m.spring(0.0, "default");
        m.set_target(id, 1.0);
        m.advance(1.0 / 60.0);
        let v = m.velocity(id);
        m.set_target(id, 0.0);
        assert_eq!(m.velocity(id), v);
    }

    #[test]
    fn reduced_motion_never_overshoots() {
        let mut m = Motion::new(true);
        let id = m.spring(0.0, "expressive");
        m.set_target(id, 1.0);
        let mut max: f64 = 0.0;
        for _ in 0..600 {
            m.advance(1.0 / 60.0);
            max = max.max(m.value(id));
        }
        assert!(max <= 1.0 + 1e-9);
    }

    #[test]
    fn handles_are_reused_and_unknown_handles_are_harmless() {
        let mut m = Motion::new(false);
        let a = m.spring(1.0, "fast");
        m.release(a);
        m.release(a);
        let b = m.spring(2.0, "gentle_spatial");
        assert_eq!(a, b);
        assert_eq!(m.value(b), 2.0);
        m.set_target(99, 5.0);
        assert_eq!(m.value(99), 0.0);
        assert!(!m.is_active(99));
    }
}
