use std::hash::Hash;

use indexmap::{IndexMap, IndexSet};

use crate::catalog::Catalog;
use crate::health::PartDiff;

/// What restoring a snapshot would change in the current catalog, part by part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SnapshotDiff {
    pub documents: PartDiff,
    pub categories: PartDiff,
    pub tags: PartDiff,
    pub contracts: PartDiff,
    pub cases: PartDiff,
    pub settings_changed: bool,
}

impl SnapshotDiff {
    #[must_use]
    pub fn between(current: &Catalog, snapshot: &Catalog) -> Self {
        Self {
            documents: map_diff(current.documents(), snapshot.documents()),
            categories: map_diff(current.categories(), snapshot.categories()),
            tags: set_diff(current.tags(), snapshot.tags()),
            contracts: map_diff(current.contracts(), snapshot.contracts()),
            cases: map_diff(current.cases(), snapshot.cases()),
            settings_changed: current.settings() != snapshot.settings(),
        }
    }

    /// Restoring would change nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        [self.documents, self.categories, self.tags, self.contracts, self.cases].iter().all(|part| part.is_empty())
            && !self.settings_changed
    }
}

fn map_diff<K: Hash + Eq, V: PartialEq>(current: &IndexMap<K, V>, snapshot: &IndexMap<K, V>) -> PartDiff {
    PartDiff {
        added: snapshot.keys().filter(|key| !current.contains_key(*key)).count(),
        removed: current.keys().filter(|key| !snapshot.contains_key(*key)).count(),
        changed: snapshot.iter().filter(|(key, value)| current.get(*key).is_some_and(|now| now != *value)).count(),
    }
}

fn set_diff<T: Hash + Eq>(current: &IndexSet<T>, snapshot: &IndexSet<T>) -> PartDiff {
    PartDiff { added: snapshot.difference(current).count(), removed: current.difference(snapshot).count(), changed: 0 }
}
