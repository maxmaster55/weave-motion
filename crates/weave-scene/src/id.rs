//! Stable element identity.
//!
//! UI trees are rebuilt constantly: a list reorders, a screen changes, a
//! widget is recycled. Weave therefore never identifies an element by its
//! position in a tree or by an index. Instead the application provides a
//! **stable key** — a document id, a cell address, a route name — and the
//! scene interns it into an [`ElementId`] that survives every rebuild.

use alloc::collections::BTreeMap;
use alloc::string::String;

/// A stable identifier for a UI element.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementId(u64);

impl ElementId {
    /// The raw identifier value.
    pub fn raw(self) -> u64 {
        self.0
    }
}

/// Interns stable keys into [`ElementId`]s.
///
/// The registry is the single place where identity is decided. Two calls
/// with the same key always return the same identifier, no matter how many
/// times the UI tree was rebuilt in between.
#[derive(Clone, Debug, Default)]
pub struct ElementRegistry {
    ids: BTreeMap<String, ElementId>,
    next_id: u64,
}

impl ElementRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the identifier for `key`, creating one when needed.
    pub fn intern(&mut self, key: &str) -> ElementId {
        if let Some(id) = self.ids.get(key) {
            return *id;
        }

        self.next_id += 1;
        let id = ElementId(self.next_id);
        self.ids.insert(String::from(key), id);
        id
    }

    /// Returns the identifier for `key`, if it was interned before.
    pub fn get(&self, key: &str) -> Option<ElementId> {
        self.ids.get(key).copied()
    }

    /// The key an identifier was interned from.
    pub fn key(&self, id: ElementId) -> Option<&str> {
        self.ids
            .iter()
            .find(|(_, value)| **value == id)
            .map(|(key, _)| key.as_str())
    }

    /// Forgets a key.
    ///
    /// The identifier is *not* reused: a late frame that still refers to the
    /// old identifier can never be confused with a new element.
    pub fn forget(&mut self, key: &str) -> Option<ElementId> {
        self.ids.remove(key)
    }

    /// Number of interned keys.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_key_always_yields_the_same_identifier() {
        let mut registry = ElementRegistry::new();
        let first = registry.intern("document.42");
        let second = registry.intern("document.42");
        assert_eq!(first, second);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn different_keys_yield_different_identifiers() {
        let mut registry = ElementRegistry::new();
        assert_ne!(registry.intern("a"), registry.intern("b"));
    }

    #[test]
    fn identifiers_are_never_reused() {
        let mut registry = ElementRegistry::new();
        let first = registry.intern("a");
        registry.forget("a");
        let second = registry.intern("a");
        assert_ne!(first, second);
    }

    #[test]
    fn keys_can_be_looked_up() {
        let mut registry = ElementRegistry::new();
        let id = registry.intern("document.42");
        assert_eq!(registry.get("document.42"), Some(id));
        assert_eq!(registry.key(id), Some("document.42"));
    }
}
