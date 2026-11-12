use chrono::{Days, NaiveDate};

use crate::cases::{Case, CaseEntry, CaseNote, CaseStatus, EntryDirection, FollowUp, FollowUpId};
use crate::document::DocumentId;

/// The changes the user makes to a case. Each returns the changed case; the catalog saves it with `save_case`.
impl Case {
    /// The case with `document` in it, `direction` on `date`; a document already in the case takes the new
    /// direction and date. An answer (a received document) ends the waiting: a waiting case opens again.
    #[must_use]
    pub fn with_entry(&self, document: &DocumentId, direction: EntryDirection, date: NaiveDate) -> Self {
        let mut case = self.clone();
        let entry = CaseEntry { document_id: document.clone(), direction, date };
        match case.entries.iter_mut().find(|entry| &entry.document_id == document) {
            Some(existing) => *existing = entry,
            None => case.entries.push(entry),
        }
        if direction == EntryDirection::Received && case.status == CaseStatus::Waiting {
            case.status = CaseStatus::Open;
            case.waiting_since = None;
        }
        case
    }

    /// The case without `document`; the document itself stays in the library.
    #[must_use]
    pub fn without_document(&self, document: &DocumentId) -> Self {
        self.retaining_documents(&|id: &DocumentId| id != document)
    }

    /// The case in `status` from `today`: waiting counts from today (or keeps counting when it already waits),
    /// done records the day it was closed, open clears both.
    #[must_use]
    pub fn with_status(&self, status: CaseStatus, today: NaiveDate) -> Self {
        let mut case = self.clone();
        case.status = status;
        case.waiting_since = match status {
            CaseStatus::Waiting => self.waiting_since.filter(|_| self.status == CaseStatus::Waiting).or(Some(today)),
            CaseStatus::Open | CaseStatus::Done => None,
        };
        case.closed_on = match status {
            CaseStatus::Done => self.closed_on.filter(|_| self.status == CaseStatus::Done).or(Some(today)),
            CaseStatus::Open | CaseStatus::Waiting => None,
        };
        case
    }

    #[must_use]
    pub fn with_follow_up(&self, follow_up: FollowUp) -> Self {
        let mut case = self.clone();
        case.follow_ups.push(follow_up);
        case
    }

    #[must_use]
    pub fn with_note(&self, note: CaseNote) -> Self {
        let mut case = self.clone();
        case.notes.push(note);
        case
    }

    /// The case with the follow-up marked done; `None` when the case has no such follow-up.
    #[must_use]
    pub fn completing(&self, follow_up: &FollowUpId) -> Option<Self> {
        self.changing_follow_up(follow_up, |follow_up| follow_up.done = true)
    }

    /// The case with the follow-up moved `days` after its date, or after `today` when that date has passed;
    /// `None` when the case has no such follow-up.
    #[must_use]
    pub fn snoozing(&self, follow_up: &FollowUpId, days: u16, today: NaiveDate) -> Option<Self> {
        self.changing_follow_up(follow_up, |follow_up| {
            let from = follow_up.date.max(today);
            follow_up.date = from.checked_add_days(Days::new(u64::from(days))).unwrap_or(from);
        })
    }

    /// The open follow-ups, earliest first.
    pub fn open_follow_ups(&self) -> impl Iterator<Item = &FollowUp> {
        let mut open: Vec<&FollowUp> = self.follow_ups.iter().filter(|follow_up| !follow_up.done).collect();
        open.sort_by_key(|follow_up| follow_up.date);
        open.into_iter()
    }

    fn changing_follow_up(&self, id: &FollowUpId, change: impl FnOnce(&mut FollowUp)) -> Option<Self> {
        let mut case = self.clone();
        change(case.follow_ups.iter_mut().find(|follow_up| &follow_up.id == id)?);
        Some(case)
    }
}
