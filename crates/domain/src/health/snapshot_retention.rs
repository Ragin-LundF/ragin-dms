use std::collections::HashSet;

use chrono::{DateTime, Datelike, TimeDelta, Utc};

/// Which catalog snapshots to keep: the newest 20, plus the newest of each day of the last 30 days, plus the newest
/// of each week of the last year.
pub struct SnapshotRetention;

const NEWEST: usize = 20;

impl SnapshotRetention {
    /// The indexes of `taken_at` (any order) whose snapshots can be removed at `now`.
    #[must_use]
    pub fn to_remove(taken_at: &[DateTime<Utc>], now: DateTime<Utc>) -> Vec<usize> {
        let mut newest_first: Vec<usize> = (0..taken_at.len()).collect();
        newest_first.sort_by(|first, second| taken_at[*second].cmp(&taken_at[*first]));
        let mut kept: HashSet<usize> = newest_first.iter().take(NEWEST).copied().collect();
        let mut days = HashSet::new();
        let mut weeks = HashSet::new();
        for index in &newest_first {
            let at = taken_at[*index];
            let age = now - at;
            if age <= TimeDelta::days(30) && days.insert(at.date_naive()) {
                kept.insert(*index);
            }
            let week = at.iso_week();
            if age <= TimeDelta::days(365) && weeks.insert((week.year(), week.week())) {
                kept.insert(*index);
            }
        }
        let mut removed: Vec<usize> = (0..taken_at.len()).filter(|index| !kept.contains(index)).collect();
        removed.sort_unstable();
        removed
    }
}
