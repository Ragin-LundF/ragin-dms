use dms_domain::document::DocumentId;
use dms_domain::split::SplitPart;

/// The cuts the user confirmed: the parts in page order, with the AI's values where a part starts where the AI's
/// part started, the `model` that proposed them, and whether the original goes to the trash afterwards.
#[derive(Debug, Clone, PartialEq)]
pub struct ApplySplitInput {
    pub document_id: DocumentId,
    pub parts: Vec<SplitPart>,
    pub model: String,
    pub trash_original: bool,
    pub confirmed: bool,
}
