use dms_domain::cases::EntryDirection;
use dms_domain::document::DocumentId;

use crate::cases::NewFollowUp;

/// A case to open: its title, the correspondent, the documents it starts with and its first follow-ups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCase {
    pub title: String,
    pub correspondent: Option<String>,
    pub documents: Vec<(DocumentId, EntryDirection)>,
    pub follow_ups: Vec<NewFollowUp>,
}
