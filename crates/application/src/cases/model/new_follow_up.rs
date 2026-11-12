use chrono::NaiveDate;
use dms_domain::cases::FollowUpSource;

/// A follow-up to add; it gets its id when it is added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewFollowUp {
    pub text: String,
    pub date: NaiveDate,
    pub source: FollowUpSource,
}
