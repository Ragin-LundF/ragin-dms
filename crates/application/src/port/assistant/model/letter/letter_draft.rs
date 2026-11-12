/// A letter's subject line and body, from the salutation to the signature; the user edits both before saving.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LetterDraft {
    pub subject: String,
    pub body: String,
}
