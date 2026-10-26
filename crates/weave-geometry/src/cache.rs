//! Caching of morph plans.
//!
//! Computing a correspondence is expensive: both contours are flattened,
//! resampled and matched. The result only depends on the two shapes, so it
//! is cached by the *versions* of the shapes. Applications bump the
//! version whenever they change a shape, which invalidates the entry.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::morph::MorphPlan;
use crate::path::Path;

/// The key of a cached morph plan.
///
/// The two version numbers identify the shapes; they are chosen by the
/// application, which knows when a shape changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MorphCacheKey {
    /// The version of the source shape.
    pub from: u64,
    /// The version of the destination shape.
    pub to: u64,
    /// The number of samples used for the correspondence.
    pub samples: u32,
}

impl MorphCacheKey {
    /// Creates a key.
    pub const fn new(from: u64, to: u64, samples: u32) -> Self {
        Self { from, to, samples }
    }
}

/// A cache of morph plans.
#[derive(Clone, Debug, Default)]
pub struct MorphCache {
    entries: BTreeMap<MorphCacheKey, MorphPlan>,
    hits: u64,
    misses: u64,
}

impl MorphCache {
    /// Creates an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached plan for `key`, if there is one.
    pub fn get(&mut self, key: &MorphCacheKey) -> Option<&MorphPlan> {
        if self.entries.contains_key(key) {
            self.hits += 1;
            self.entries.get(key)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Stores a plan.
    pub fn insert(&mut self, key: MorphCacheKey, plan: MorphPlan) {
        self.entries.insert(key, plan);
    }

    /// Returns the plan for `key`, building it when it is missing.
    ///
    /// Returns `None` when the two paths cannot be morphed safely.
    pub fn get_or_build(
        &mut self,
        key: MorphCacheKey,
        from: &Path,
        to: &Path,
    ) -> Option<&MorphPlan> {
        if self.entries.contains_key(&key) {
            self.hits += 1;
            return self.entries.get(&key);
        }

        self.misses += 1;
        let plan = MorphPlan::build_with_samples(from, to, key.samples as usize)?;
        self.entries.insert(key, plan);
        self.entries.get(&key)
    }

    /// Removes the entry for `key`.
    pub fn remove(&mut self, key: &MorphCacheKey) -> bool {
        self.entries.remove(key).is_some()
    }

    /// Removes every entry.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// The number of cached plans.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The number of cache hits.
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// The number of cache misses.
    pub fn misses(&self) -> u64 {
        self.misses
    }

    /// The cached keys, in order.
    pub fn keys(&self) -> Vec<MorphCacheKey> {
        self.entries.keys().copied().collect()
    }
}
