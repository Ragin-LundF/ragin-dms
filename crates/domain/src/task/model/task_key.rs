use std::fmt;

/// Stable identity of an open task, derived from what the task is about. The same change found by two scans
/// yields the same key, so a dismissed task stays dismissed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskKey(String);

impl TaskKey {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TaskKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
