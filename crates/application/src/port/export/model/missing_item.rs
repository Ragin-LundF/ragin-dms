/// One line of a tax export's "Missing items" sheet: a checklist item that has no document yet.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MissingItem {
    pub area: String,
    pub label: String,
    pub from: String,
    pub state: String,
    pub note: String,
}
