//! The high level engine API.
//!
//! [`MotionEngine`] is the façade most applications use: it owns a
//! [`Scheduler`], applies the active [`MotionPolicy`] and offers a handful
//! of typed convenience methods.
//!
//! ```
//! use weaver_motion::{MotionEngine, MotionToken};
//!
//! let mut engine = MotionEngine::new();
//! let scale = engine.spring("toolbar.create.scale", 1.0, MotionToken::FastSpatial);
//!
//! engine.retarget(scale, 1.06);
//! let snapshot = engine.tick(1.0 / 60.0);
//!
//! assert!(snapshot.needs_frame);
//! ```

use alloc::boxed::Box;
use alloc::string::String;

use crate::policy::MotionPolicy;
use crate::scheduler::{AnimationId, MotionSnapshot, Scheduler};
use crate::snapshot::ValueSnapshot;
use crate::spring::SpringConfig;
use crate::tokens::MotionToken;
use crate::tween::Tween;
use crate::tween::TweenConfig;
use crate::types::Vec2;
use crate::value::Animated;

/// The application facing motion engine.
#[derive(Debug, Default)]
pub struct MotionEngine {
    scheduler: Scheduler,
    policy: MotionPolicy,
}

impl MotionEngine {
    /// Creates an engine with the default policy.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an engine with an explicit policy.
    pub fn with_policy(policy: MotionPolicy) -> Self {
        Self {
            scheduler: Scheduler::new(),
            policy,
        }
    }

    /// The active motion policy.
    pub fn policy(&self) -> MotionPolicy {
        self.policy
    }

    /// Changes the motion policy.
    ///
    /// Existing animations keep their configuration until they are
    /// re-created; new animations pick up the new policy immediately.
    pub fn set_policy(&mut self, policy: MotionPolicy) {
        self.policy = policy;
    }

    /// Registers a scalar spring under `key`.
    ///
    /// The spring starts at rest at `initial`. Use [`MotionEngine::retarget`]
    /// to move it. If the key already holds a spring, its current value and
    /// velocity are carried over so that repeated calls stay continuous.
    pub fn spring(
        &mut self,
        key: impl Into<String>,
        initial: f64,
        token: MotionToken,
    ) -> AnimationId {
        let config = token
            .spring_config()
            .expect("`MotionToken::Effects` has no spring flavour");
        let config = self.policy.spring_config(config);
        self.insert_spring(key.into(), initial, config)
    }

    /// Registers a vector spring under `key`.
    pub fn spring_vec2(
        &mut self,
        key: impl Into<String>,
        initial: Vec2,
        token: MotionToken,
    ) -> AnimationId {
        let config = token
            .spring_config()
            .expect("`MotionToken::Effects` has no spring flavour");
        let config = self.policy.spring_config(config);
        self.insert_spring_vec2(key.into(), initial, config)
    }

    /// Registers a scalar spring with an explicit configuration.
    pub fn spring_with_config(
        &mut self,
        key: impl Into<String>,
        initial: f64,
        config: SpringConfig,
    ) -> AnimationId {
        let config = self.policy.spring_config(config);
        self.insert_spring(key.into(), initial, config)
    }

    /// Registers a scalar tween under `key`.
    pub fn tween(
        &mut self,
        key: impl Into<String>,
        initial: f64,
        token: MotionToken,
    ) -> AnimationId {
        let config = token
            .tween_config()
            .expect("spatial tokens have no tween flavour");
        let config = self.policy.tween_config(config);
        self.insert_tween(key.into(), initial, config)
    }

    /// Registers a scalar tween that is already finished.
    ///
    /// Use it for properties that are only animated when they change, so
    /// an idle scene stays idle.
    pub fn tween_resting(
        &mut self,
        key: impl Into<String>,
        value: f64,
        token: MotionToken,
    ) -> AnimationId {
        let config = token
            .tween_config()
            .expect("spatial tokens have no tween flavour");
        let config = self.policy.tween_config(config);
        self.insert_tween(key.into(), value, config)
    }

    /// Registers a scalar tween with an explicit configuration.
    pub fn tween_with_config(
        &mut self,
        key: impl Into<String>,
        initial: f64,
        config: TweenConfig,
    ) -> AnimationId {
        let config = self.policy.tween_config(config);
        self.insert_tween(key.into(), initial, config)
    }

    /// Moves an animation towards a new destination.
    ///
    /// The current value **and** velocity are preserved, so interrupting an
    /// animation never restarts it. Works for springs and tweens.
    ///
    /// Under [`MotionPolicy::Off`] the value jumps to the destination
    /// immediately instead of animating.
    pub fn retarget(&mut self, id: AnimationId, target: f64) -> bool {
        if let Some(value) = self.scheduler.get_animated_mut::<f64>(id) {
            if self.policy == MotionPolicy::Off {
                value.snap(target);
            } else {
                value.set_target(target);
            }
            return true;
        }
        if let Some(value) = self.scheduler.get_tween_mut::<f64>(id) {
            value.set_target(target);
            return true;
        }
        false
    }

    /// Moves a vector animation towards a new destination.
    pub fn retarget_vec2(&mut self, id: AnimationId, target: Vec2) -> bool {
        if let Some(value) = self.scheduler.get_animated_mut::<Vec2>(id) {
            if self.policy == MotionPolicy::Off {
                value.snap(target);
            } else {
                value.set_target(target);
            }
            return true;
        }
        if let Some(value) = self.scheduler.get_tween_mut::<Vec2>(id) {
            value.set_target(target);
            return true;
        }
        false
    }

    /// The current value of an animation.
    pub fn value(&self, id: AnimationId) -> Option<ValueSnapshot> {
        self.scheduler.value(id)
    }

    /// The current value of the animation registered under `key`.
    pub fn value_by_key(&self, key: &str) -> Option<ValueSnapshot> {
        self.scheduler.value_by_key(key)
    }

    /// The identifier of the animation registered under `key`.
    pub fn id_by_key(&self, key: &str) -> Option<AnimationId> {
        self.scheduler.id_by_key(key)
    }

    /// The current value of an animation, as a scalar.
    pub fn value_f64(&self, id: AnimationId) -> Option<f64> {
        self.value(id).and_then(|value| value.as_scalar())
    }

    /// The current value of an animation, as a vector.
    pub fn value_vec2(&self, id: AnimationId) -> Option<Vec2> {
        self.value(id).and_then(|value| value.as_vector())
    }

    /// Advances the engine and returns the frame snapshot.
    pub fn tick(&mut self, timestamp: f64) -> MotionSnapshot {
        self.scheduler.tick(timestamp)
    }

    /// `true` when no animation has motion left.
    pub fn is_idle(&self) -> bool {
        self.scheduler.is_idle()
    }

    /// `true` when at least one animation still has motion left.
    pub fn needs_frame(&self) -> bool {
        self.scheduler.needs_frame()
    }

    /// Removes an animation.
    pub fn remove(&mut self, id: AnimationId) -> bool {
        self.scheduler.remove(id)
    }

    /// Removes the animation registered under `key`.
    pub fn remove_key(&mut self, key: &str) -> bool {
        self.scheduler.remove_by_key(key).is_some()
    }

    /// Read access to the scheduler.
    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    /// Mutable access to the scheduler.
    pub fn scheduler_mut(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }

    fn insert_spring(&mut self, key: String, initial: f64, config: SpringConfig) -> AnimationId {
        let carried = self
            .existing_key_id(&key)
            .and_then(|id| self.scheduler.get_animated::<f64>(id))
            .map(|animated| (animated.value(), animated.velocity(), animated.target()));

        let animated = match carried {
            Some((value, velocity, target)) => {
                Animated::with_velocity(value, velocity, target, config)
            }
            None => Animated::new(initial, initial, config),
        };

        self.scheduler.insert(Some(key), Box::new(animated))
    }

    fn insert_spring_vec2(
        &mut self,
        key: String,
        initial: Vec2,
        config: SpringConfig,
    ) -> AnimationId {
        let carried = self
            .existing_key_id(&key)
            .and_then(|id| self.scheduler.get_animated::<Vec2>(id))
            .map(|animated| (animated.value(), animated.velocity(), animated.target()));

        let animated = match carried {
            Some((value, velocity, target)) => {
                Animated::with_velocity(value, velocity, target, config)
            }
            None => Animated::new(initial, initial, config),
        };

        self.scheduler.insert(Some(key), Box::new(animated))
    }

    fn insert_tween(&mut self, key: String, initial: f64, config: TweenConfig) -> AnimationId {
        self.scheduler
            .insert(Some(key), Box::new(Tween::resting(initial, config)))
    }

    fn existing_key_id(&self, key: &str) -> Option<AnimationId> {
        self.scheduler.id_by_key(key)
    }
}
