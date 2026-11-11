//! The questions asked in this session, in memory only, as in Kotlin's `AskViewModel`.

use std::sync::Mutex;

use super::dto::AskEntryDto;
use crate::session::lock;

// shortcut: process-wide like the Kotlin view model (one window); move into AppContext if a second window comes.
static ENTRIES: Mutex<Vec<AskEntryDto>> = Mutex::new(Vec::new());

pub fn entries() -> Vec<AskEntryDto> {
    lock(&ENTRIES).clone()
}

/// Adds `entry` and returns its position.
pub fn push(entry: AskEntryDto) -> usize {
    let mut entries = lock(&ENTRIES);
    entries.push(entry);
    entries.len() - 1
}

pub fn replace(position: usize, entry: AskEntryDto) {
    if let Some(slot) = lock(&ENTRIES).get_mut(position) {
        *slot = entry;
    }
}

/// A stopped or failed question must not keep waiting for an answer that never comes.
pub fn stop_waiting() {
    for entry in lock(&ENTRIES).iter_mut().filter(|entry| entry.asking) {
        entry.asking = false;
        entry.note = Some("Stopped before an answer".to_owned());
    }
}

/// Forgets the questions when another library opens; their answers came from the other library.
pub fn reset() {
    lock(&ENTRIES).clear();
}
