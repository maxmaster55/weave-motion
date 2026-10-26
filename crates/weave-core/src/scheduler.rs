//! The animation registry and frame scheduler.
//!
//! The scheduler owns every animated value of an application, advances them
//! all from a single monotonic timestamp and hands adapters an immutable
//! [`MotionSnapshot`]. It never asks for a frame while everything is at
//! rest.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::animatable::Animatable;
use crate::clock::{MotionClock, Tick};
use crate::snapshot::ValueSnapshot;
use crate::tween::Tween;
use crate::value::Animated;

/// Identifier of an animation.
///
/// Identifiers are stable for the lifetime of the animation and are never
/// reused, so a late frame cannot resurrect a removed animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnimationId(u64);

impl AnimationId {
    /// The raw identifier value.
    pub fn raw(self) -> u64 {
        self.0
    }
}

/// The kind of primitive driving an animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimationKind {
    /// A spring driven value.
    Spring,
    /// A fixed duration tween.
    Tween,
}

/// Events reported by a scheduler tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationEvent {
    /// The animation reached its destination and stopped.
    Settled(AnimationId),
    /// The animation was removed from the registry.
    Removed(AnimationId),
}

/// A value that can live inside the scheduler.
pub trait AnimatedValue: Any + Send + Sync {
    /// The kind of primitive driving the value.
    fn kind(&self) -> AnimationKind;
    /// The current value, in a renderer-agnostic form.
    fn snapshot(&self) -> ValueSnapshot;
    /// Whether the value still has motion left.
    fn is_active(&self) -> bool;
    /// Advances the value, returning `true` while it is still moving.
    fn advance(&mut self, dt: f64) -> bool;
    /// Jumps to the destination immediately.
    fn settle(&mut self);
    /// Downcast helper.
    fn as_any(&self) -> &dyn Any;
    /// Mutable downcast helper.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> AnimatedValue for Animated<T>
where
    T: Animatable + Send + Sync,
{
    fn kind(&self) -> AnimationKind {
        AnimationKind::Spring
    }

    fn snapshot(&self) -> ValueSnapshot {
        self.value().snapshot()
    }

    fn is_active(&self) -> bool {
        Animated::is_active(self)
    }

    fn advance(&mut self, dt: f64) -> bool {
        Animated::advance(self, dt)
    }

    fn settle(&mut self) {
        Animated::settle(self);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<T> AnimatedValue for Tween<T>
where
    T: Animatable + Send + Sync,
{
    fn kind(&self) -> AnimationKind {
        AnimationKind::Tween
    }

    fn snapshot(&self) -> ValueSnapshot {
        self.value().snapshot()
    }

    fn is_active(&self) -> bool {
        Tween::is_active(self)
    }

    fn advance(&mut self, dt: f64) -> bool {
        Tween::advance(self, dt)
    }

    fn settle(&mut self) {
        self.snap(self.target());
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// One animation inside a [`MotionSnapshot`].
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationSnapshot {
    /// The animation's identifier.
    pub id: AnimationId,
    /// The optional key the animation was registered under.
    pub key: Option<String>,
    /// The kind of primitive driving the animation.
    pub kind: AnimationKind,
    /// The current value.
    pub value: ValueSnapshot,
    /// Whether the animation is still moving.
    pub active: bool,
}

/// An immutable view of the animation state after a tick.
///
/// Adapters consume snapshots instead of reaching into the animation state:
/// that keeps the core decoupled and makes a frame reproducible.
#[derive(Clone, Debug, Default)]
pub struct MotionSnapshot {
    /// The timestamp the frame was produced for.
    pub timestamp: f64,
    /// Seconds elapsed since the previous frame.
    pub dt: f64,
    /// One entry per registered animation, in registration order.
    pub entries: Vec<AnimationSnapshot>,
    /// Events produced during the tick.
    pub events: Vec<AnimationEvent>,
    /// Whether the application should request another frame.
    pub needs_frame: bool,
}

impl MotionSnapshot {
    /// Looks an animation up by identifier.
    pub fn get(&self, id: AnimationId) -> Option<&AnimationSnapshot> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    /// Looks an animation up by key.
    pub fn get_by_key(&self, key: &str) -> Option<&AnimationSnapshot> {
        self.entries
            .iter()
            .find(|entry| entry.key.as_deref() == Some(key))
    }

    /// Iterates over the entries.
    pub fn iter(&self) -> core::slice::Iter<'_, AnimationSnapshot> {
        self.entries.iter()
    }
}

struct Entry {
    id: AnimationId,
    key: Option<String>,
    value: Box<dyn AnimatedValue>,
}

/// Registry of animations plus the frame scheduler that drives them.
///
/// ```
/// use weave_core::{MotionToken, Scheduler, ValueSnapshot};
///
/// let mut scheduler = Scheduler::new();
/// scheduler.insert(
///     Some("scale".into()),
///     Box::new(weave_core::Animated::new(
///         1.0_f64,
///         1.06,
///         MotionToken::FastSpatial.spring_config().unwrap(),
///     )),
/// );
///
/// assert!(scheduler.needs_frame());
/// let snapshot = scheduler.tick(1.0 / 60.0);
/// assert!(snapshot.entries.len() == 1);
/// ```
#[derive(Default)]
pub struct Scheduler {
    next_id: u64,
    entries: Vec<Entry>,
    index: BTreeMap<u64, usize>,
    keys: BTreeMap<String, AnimationId>,
    clock: MotionClock,
    pending_events: Vec<AnimationEvent>,
}

impl core::fmt::Debug for Scheduler {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Scheduler")
            .field("animations", &self.entries.len())
            .field("clock", &self.clock)
            .finish()
    }
}

impl Scheduler {
    /// Creates an empty scheduler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a scheduler with an explicit clock.
    pub fn with_clock(clock: MotionClock) -> Self {
        Self {
            clock,
            ..Self::default()
        }
    }

    /// The clock driving the scheduler.
    pub fn clock(&self) -> &MotionClock {
        &self.clock
    }

    /// Mutable access to the clock, e.g. to change the gap policy.
    pub fn clock_mut(&mut self) -> &mut MotionClock {
        &mut self.clock
    }

    /// Registers an animation, optionally under a key.
    ///
    /// A key can only be used by one animation at a time: registering a
    /// second animation under an existing key removes the first one, so a
    /// fast sequence of toggles can never leave orphaned animators behind.
    pub fn insert(&mut self, key: Option<String>, value: Box<dyn AnimatedValue>) -> AnimationId {
        if let Some(key) = &key {
            if let Some(previous) = self.keys.get(key).copied() {
                self.remove(previous);
            }
        }

        self.next_id += 1;
        let id = AnimationId(self.next_id);

        if let Some(key) = &key {
            self.keys.insert(key.clone(), id);
        }

        self.entries.push(Entry {
            id,
            key: key.clone(),
            value,
        });
        self.index.insert(id.0, self.entries.len() - 1);

        id
    }

    /// Removes an animation.
    pub fn remove(&mut self, id: AnimationId) -> bool {
        let Some(position) = self.index.get(&id.0).copied() else {
            return false;
        };

        let entry = self.entries.remove(position);
        if let Some(key) = &entry.key {
            if self.keys.get(key) == Some(&id) {
                self.keys.remove(key);
            }
        }
        self.pending_events.push(AnimationEvent::Removed(id));
        self.reindex();
        true
    }

    /// Removes the animation registered under `key`.
    pub fn remove_by_key(&mut self, key: &str) -> Option<AnimationId> {
        let id = self.keys.get(key).copied()?;
        self.remove(id);
        Some(id)
    }

    /// The identifier of the animation registered under `key`.
    pub fn id_by_key(&self, key: &str) -> Option<AnimationId> {
        self.keys.get(key).copied()
    }

    /// The key an animation was registered under.
    pub fn key_of(&self, id: AnimationId) -> Option<&str> {
        let position = *self.index.get(&id.0)?;
        self.entries.get(position)?.key.as_deref()
    }

    /// Whether an animation with that identifier is registered.
    pub fn contains(&self, id: AnimationId) -> bool {
        self.index.contains_key(&id.0)
    }

    /// Number of registered animations.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no animation is registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Identifiers of all registered animations, in registration order.
    pub fn ids(&self) -> impl Iterator<Item = AnimationId> + '_ {
        self.entries.iter().map(|entry| entry.id)
    }

    /// `true` when no animation has motion left.
    pub fn is_idle(&self) -> bool {
        !self.needs_frame()
    }

    /// `true` when at least one animation still has motion left.
    pub fn needs_frame(&self) -> bool {
        self.entries.iter().any(|entry| entry.value.is_active())
    }

    /// The current value of an animation.
    pub fn value(&self, id: AnimationId) -> Option<ValueSnapshot> {
        let position = *self.index.get(&id.0)?;
        self.entries
            .get(position)
            .map(|entry| entry.value.snapshot())
    }

    /// The current value of the animation registered under `key`.
    pub fn value_by_key(&self, key: &str) -> Option<ValueSnapshot> {
        let id = self.keys.get(key).copied()?;
        self.value(id)
    }

    /// Typed access to a spring value.
    pub fn get_animated<T>(&self, id: AnimationId) -> Option<&Animated<T>>
    where
        T: Animatable + Send + Sync,
    {
        let entry = self.entries.get(self.index.get(&id.0).copied()?)?;
        entry.value.as_any().downcast_ref::<Animated<T>>()
    }

    /// Mutable typed access to a spring value.
    pub fn get_animated_mut<T>(&mut self, id: AnimationId) -> Option<&mut Animated<T>>
    where
        T: Animatable + Send + Sync,
    {
        let position = *self.index.get(&id.0)?;
        let entry = self.entries.get_mut(position)?;
        entry.value.as_any_mut().downcast_mut::<Animated<T>>()
    }

    /// Typed access to a tween value.
    pub fn get_tween<T>(&self, id: AnimationId) -> Option<&Tween<T>>
    where
        T: Animatable + Send + Sync,
    {
        let entry = self.entries.get(self.index.get(&id.0).copied()?)?;
        entry.value.as_any().downcast_ref::<Tween<T>>()
    }

    /// Mutable typed access to a tween value.
    pub fn get_tween_mut<T>(&mut self, id: AnimationId) -> Option<&mut Tween<T>>
    where
        T: Animatable + Send + Sync,
    {
        let position = *self.index.get(&id.0)?;
        let entry = self.entries.get_mut(position)?;
        entry.value.as_any_mut().downcast_mut::<Tween<T>>()
    }

    /// Advances every animation and returns the resulting snapshot.
    ///
    /// The snapshot reports whether another frame is needed: when nothing
    /// moves any more, the application can stop scheduling frames.
    pub fn tick(&mut self, timestamp: f64) -> MotionSnapshot {
        let Tick { dt, snapped } = self.clock.tick(timestamp);
        let mut events = core::mem::take(&mut self.pending_events);

        if snapped {
            for entry in &mut self.entries {
                if entry.value.is_active() {
                    entry.value.settle();
                    events.push(AnimationEvent::Settled(entry.id));
                }
            }
        } else if dt > 0.0 {
            for entry in &mut self.entries {
                if entry.value.is_active() && !entry.value.advance(dt) {
                    events.push(AnimationEvent::Settled(entry.id));
                }
            }
        }

        let entries = self
            .entries
            .iter()
            .map(|entry| AnimationSnapshot {
                id: entry.id,
                key: entry.key.clone(),
                kind: entry.value.kind(),
                value: entry.value.snapshot(),
                active: entry.value.is_active(),
            })
            .collect();

        MotionSnapshot {
            timestamp,
            dt,
            entries,
            events,
            needs_frame: self.needs_frame(),
        }
    }

    /// Pauses the clock: animations stop advancing.
    pub fn pause(&mut self) {
        self.clock.pause();
    }

    /// Resumes the clock.
    pub fn resume(&mut self) {
        self.clock.resume();
    }

    /// Whether the clock is paused.
    pub fn is_paused(&self) -> bool {
        self.clock.is_paused()
    }

    fn reindex(&mut self) {
        self.index.clear();
        for (position, entry) in self.entries.iter().enumerate() {
            self.index.insert(entry.id.0, position);
        }
    }
}
