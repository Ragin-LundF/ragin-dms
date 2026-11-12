use chrono::NaiveDate;

use crate::DomainError;
use crate::cases::{CaseEntry, CaseId, CaseNote, CaseStatus, FollowUp};
use crate::document::DocumentId;
use crate::error::{require, require_distinct, require_none_or_not_blank, require_not_blank};

/// A matter with a correspondent: its documents in and out, notes, and follow-ups. A document may belong to
/// several cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Case {
    pub id: CaseId,
    pub title: String,
    pub correspondent: Option<String>,
    pub status: CaseStatus,
    pub opened_on: NaiveDate,
    pub closed_on: Option<NaiveDate>,
    pub waiting_since: Option<NaiveDate>,
    pub remind_after_days: u16,
    pub entries: Vec<CaseEntry>,
    pub notes: Vec<CaseNote>,
    pub follow_ups: Vec<FollowUp>,
}

impl Case {
    /// Days a waiting case waits before the dashboard reminds the user.
    pub const DEFAULT_REMIND_AFTER_DAYS: u16 = 21;

    /// An open case without documents, notes or follow-ups.
    ///
    /// # Errors
    /// Fails for a blank title.
    pub fn new(id: CaseId, title: impl Into<String>, opened_on: NaiveDate) -> Result<Self, DomainError> {
        let case = Self {
            id,
            title: title.into(),
            correspondent: None,
            status: CaseStatus::Open,
            opened_on,
            closed_on: None,
            waiting_since: None,
            remind_after_days: Self::DEFAULT_REMIND_AFTER_DAYS,
            entries: Vec::new(),
            notes: Vec::new(),
            follow_ups: Vec::new(),
        };
        case.validate()?;
        Ok(case)
    }

    /// # Errors
    /// Fails for a blank title or correspondent, a reminder after 0 days, two follow-ups with the same id, or an
    /// invalid note or follow-up.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_not_blank(&self.title, "Case title")?;
        require_none_or_not_blank(self.correspondent.as_deref(), "Correspondent")?;
        require(self.remind_after_days > 0, || "A case reminds after at least 1 day".to_owned())?;
        require_distinct(self.follow_ups.iter().map(|follow_up| &follow_up.id), || {
            "Two follow-ups share an id".to_owned()
        })?;
        self.notes.iter().try_for_each(CaseNote::validate)?;
        self.follow_ups.iter().try_for_each(FollowUp::validate)
    }

    pub fn referenced_documents(&self) -> impl Iterator<Item = &DocumentId> {
        self.entries.iter().map(|entry| &entry.document_id)
    }

    /// The case without the entries of the documents `keep` refuses.
    #[must_use]
    pub fn retaining_documents(&self, keep: &impl Fn(&DocumentId) -> bool) -> Self {
        let mut case = self.clone();
        case.entries.retain(|entry| keep(&entry.document_id));
        case
    }
}
