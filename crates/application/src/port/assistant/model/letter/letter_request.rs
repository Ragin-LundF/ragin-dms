use crate::port::assistant::{LetterKind, LetterLanguage, LetterParty};

/// What the AI needs to draft a letter: the facts the feature knows (such as a customer number or a cancel-by date)
/// and the texts of the source documents, already cut to the AI's input budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetterRequest {
    pub kind: LetterKind,
    pub sender: Option<LetterParty>,
    pub recipient: LetterParty,
    pub subject_facts: Vec<String>,
    pub source_texts: Vec<String>,
    pub language: LetterLanguage,
}
