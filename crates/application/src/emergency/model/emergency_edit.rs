use dms_domain::emergency::{EmergencyEntry, EmergencyGroup, EmergencyItemId};

/// One change of the emergency folder the user made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmergencyEdit {
    /// Adds an included item; a document or contract that is already in the folder is not added twice.
    Add {
        group: EmergencyGroup,
        entry: EmergencyEntry,
    },
    Remove(EmergencyItemId),
    SetIncluded {
        id: EmergencyItemId,
        included: bool,
    },
    SetIntro(Option<String>),
    SetRefreshMonths(u8),
}
