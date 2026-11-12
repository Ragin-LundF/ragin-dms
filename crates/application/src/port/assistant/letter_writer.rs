use crate::port::assistant::{LetterAnswer, LetterRequest};

/// Lets the AI draft a letter. The draft is only shown to the user, who edits it before anything is saved.
pub trait LetterWriter: Send + Sync {
    fn draft(&self, request: &LetterRequest) -> LetterAnswer;
}
