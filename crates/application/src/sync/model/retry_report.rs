use std::ops::Add;

/// The result of retrying the journal. `dropped` operations had become pointless, for example a copy of a file
/// that no longer exists in the primary folder; the next scan shows what is left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RetryReport {
    pub succeeded: usize,
    pub dropped: usize,
    pub still_pending: usize,
}

impl Add for RetryReport {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            succeeded: self.succeeded + other.succeeded,
            dropped: self.dropped + other.dropped,
            still_pending: self.still_pending + other.still_pending,
        }
    }
}
