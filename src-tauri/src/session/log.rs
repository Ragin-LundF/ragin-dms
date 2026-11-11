use std::collections::VecDeque;
use std::sync::Mutex;

use super::{LogEntry, lock};

/// What the app did in this session, newest last, for the log panel. Keeps the last [`Self::CAPACITY`] lines;
/// nothing is written to disk.
#[derive(Default)]
pub struct SessionLog {
    entries: Mutex<VecDeque<LogEntry>>,
}

impl SessionLog {
    pub const CAPACITY: usize = 2_000;

    pub fn info(&self, text: impl Into<String>) -> LogEntry {
        self.add(text.into(), false)
    }

    pub fn error(&self, text: impl Into<String>) -> LogEntry {
        self.add(text.into(), true)
    }

    pub fn entries(&self) -> Vec<LogEntry> {
        lock(&self.entries).iter().cloned().collect()
    }

    pub fn clear(&self) {
        lock(&self.entries).clear();
    }

    fn add(&self, text: String, is_error: bool) -> LogEntry {
        let entry = LogEntry { at: chrono::Utc::now().to_rfc3339(), text, is_error };
        let mut entries = lock(&self.entries);
        if entries.len() == Self::CAPACITY {
            entries.pop_front();
        }
        entries.push_back(entry.clone());
        entry
    }
}
