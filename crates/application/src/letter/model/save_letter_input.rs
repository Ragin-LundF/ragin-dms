use dms_domain::document::DocumentId;
use dms_domain::library::RelativePath;

use crate::port::assistant::{LetterLanguage, LetterParty};

/// The letter as the user edited it, the folder to save it in, and the source documents it gets linked to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveLetterInput {
    pub document_ids: Vec<DocumentId>,
    pub folder: RelativePath,
    pub sender: Option<LetterParty>,
    pub recipient: LetterParty,
    pub subject: String,
    pub body: String,
    pub language: LetterLanguage,
}
