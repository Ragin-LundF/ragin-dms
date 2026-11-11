//! Takes over the fields only newer versions know from the entries of the file that is replaced.

use std::collections::HashMap;
use std::hash::Hash;

use crate::catalog::dto::ExtensibleDto;

/// Adds to each of `entries` the unknown fields of the entry in `previous` with the same key; the last of
/// several previous entries with one key wins, as Kotlin's `associateBy`.
pub fn carry_all<T: ExtensibleDto, K: Eq + Hash>(entries: &mut [T], previous: &[T], key_of: impl Fn(&T) -> K) {
    let previous_by_key: HashMap<K, &T> = previous.iter().map(|entry| (key_of(entry), entry)).collect();
    for entry in entries {
        carry(entry, previous_by_key.get(&key_of(entry)).copied());
    }
}

/// Adds the unknown fields of `previous` to `entry`.
pub fn carry<T: ExtensibleDto>(entry: &mut T, previous: Option<&T>) {
    if let Some(previous) = previous {
        entry.unknown_mut().extend(previous.unknown().iter().map(|(key, value)| (key.clone(), value.clone())));
    }
}

pub fn has_any<T: ExtensibleDto>(entry: Option<&T>) -> bool {
    entry.is_some_and(|entry| !entry.unknown().is_empty())
}
