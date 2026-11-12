use crate::DomainError;
use crate::document::DocumentId;
use crate::error::{require_none_or_not_blank, require_not_blank};
use crate::tax::{TaxArea, TaxItemId, TaxItemState};

/// One thing a tax return needs, such as "Lohnsteuerbescheinigung" or a sender's yearly statement, and the
/// document found for it. `added_by_user` items are never removed when the checklist is built again.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaxChecklistItem {
    pub id: TaxItemId,
    pub area: TaxArea,
    pub label: String,
    pub correspondent: Option<String>,
    pub document_id: Option<DocumentId>,
    pub state: TaxItemState,
    pub note: Option<String>,
    pub added_by_user: bool,
}

impl TaxChecklistItem {
    /// # Errors
    /// Fails for a blank label, or a blank correspondent or note.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_not_blank(&self.label, "Tax item label")?;
        require_none_or_not_blank(self.correspondent.as_deref(), "Correspondent")?;
        require_none_or_not_blank(self.note.as_deref(), "Note")
    }
}
