use crate::document::ContentHash;
use crate::emergency::{EmergencyEntry, EmergencyGroup, EmergencyItemId};

/// One item of the emergency folder. `included` items go into the overview and the encrypted ZIP;
/// `recorded_hash` is a document's content when the folder was last refreshed, to show what changed since.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmergencyItem {
    pub id: EmergencyItemId,
    pub group: EmergencyGroup,
    pub entry: EmergencyEntry,
    pub included: bool,
    pub recorded_hash: Option<ContentHash>,
}
