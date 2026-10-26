//! Choreography: dependencies between the parts of a transition.
//!
//! A coordinated transition is not a timeline of fixed delays — it is a set
//! of *dependencies*. "Reveal the title only once the panel is wide enough
//! to hold it" is a dependency; "reveal the title after 120 ms" is not,
//! because it breaks when the panel grows faster or slower than expected.
//!
//! Gates are expressed in terms of the **animated** state of an element, so
//! they hold no matter how the transition is interrupted.

use crate::id::ElementId;

/// A condition that must become true before a cue may fire.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gate {
    /// The shape morph of an element progressed past `threshold` (0..1).
    MorphProgress {
        /// The element to watch.
        element: ElementId,
        /// The threshold, between 0 and 1.
        threshold: f64,
    },
    /// The animated width of an element reached `threshold` times the width
    /// of its destination.
    WidthRatio {
        /// The element to watch.
        element: ElementId,
        /// The threshold, between 0 and 1.
        threshold: f64,
    },
    /// The animated height of an element reached `threshold` times the
    /// height of its destination.
    HeightRatio {
        /// The element to watch.
        element: ElementId,
        /// The threshold, between 0 and 1.
        threshold: f64,
    },
    /// The animated opacity of an element reached `threshold`.
    Opacity {
        /// The element to watch.
        element: ElementId,
        /// The threshold, between 0 and 1.
        threshold: f64,
    },
}

/// What happens when a gate opens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cue {
    /// Reveal the content of an element: its opacity animates to 1.
    RevealContent {
        /// The element to reveal.
        element: ElementId,
    },
    /// Hide the content of an element: its opacity animates to 0.
    HideContent {
        /// The element to hide.
        element: ElementId,
    },
    /// Animate the opacity of an element to an explicit value.
    SetOpacity {
        /// The element to change.
        element: ElementId,
        /// The value to animate to.
        value: f64,
    },
}

#[derive(Clone, Debug)]
struct Entry {
    gate: Gate,
    cue: Cue,
    fired: bool,
}

/// The list of dependencies of a transition.
#[derive(Clone, Debug, Default)]
pub struct Choreographer {
    entries: Vec<Entry>,
}

impl Choreographer {
    /// Creates an empty choreographer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a dependency.
    pub fn place(&mut self, gate: Gate, cue: Cue) {
        self.entries.push(Entry {
            gate,
            cue,
            fired: false,
        });
    }

    /// Removes every dependency.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Makes every dependency eligible again.
    ///
    /// Called when a new transition starts: the same choreography has to be
    /// able to fire again for the next transition.
    pub fn rearm(&mut self) {
        for entry in &mut self.entries {
            entry.fired = false;
        }
    }

    /// Number of registered dependencies.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no dependency is registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The dependencies that are ready to fire.
    pub fn ready(&self, satisfied: impl Fn(&Gate) -> bool) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| !entry.fired && satisfied(&entry.gate))
            .map(|(index, _)| index)
            .collect()
    }

    /// Marks a dependency as fired.
    pub fn mark_fired(&mut self, index: usize) {
        if let Some(entry) = self.entries.get_mut(index) {
            entry.fired = true;
        }
    }

    /// The cue of a dependency.
    pub fn cue(&self, index: usize) -> Option<Cue> {
        self.entries.get(index).map(|entry| entry.cue)
    }
}
