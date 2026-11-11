use crate::DomainError;
use crate::error::require;
use crate::text;

/// Someone the library receives documents from. It is recognized in a document's text by its [`Self::name`] or
/// one of its [`Self::aliases`], compared as whole words and ignoring case.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Correspondent {
    pub name: String,
    pub aliases: Vec<String>,
}

impl Correspondent {
    /// # Errors
    /// Fails for a blank name or alias.
    pub fn new(name: impl Into<String>, aliases: Vec<String>) -> Result<Self, DomainError> {
        let correspondent = Self { name: name.into(), aliases };
        correspondent.validate()?;
        Ok(correspondent)
    }

    /// # Errors
    /// Fails for a blank name or alias.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(!text::is_blank(&self.name), || "Correspondent name must not be blank".to_owned())?;
        require(!self.aliases.iter().any(|alias| text::is_blank(alias)), || "Aliases must not be blank".to_owned())
    }

    /// The name ignoring case, which identifies the correspondent.
    #[must_use]
    pub fn key(&self) -> String {
        text::lowercase(text::trim(&self.name))
    }

    /// `true` if the name or an alias appears in `text` as a whole word or phrase.
    #[must_use]
    pub fn appears_in(&self, text: &str) -> bool {
        let haystack = text::lowercase(text);
        std::iter::once(&self.name).chain(&self.aliases).any(|phrase| contains_phrase(&haystack, phrase))
    }
}

fn contains_phrase(haystack: &str, phrase: &str) -> bool {
    let needle = text::lowercase(text::trim(phrase));
    let mut from = 0;
    while let Some(offset) = haystack.get(from..).and_then(|rest| rest.find(&needle)) {
        let index = from + offset;
        let before = haystack[..index].chars().next_back();
        let after = haystack[index + needle.len()..].chars().next();
        if !is_word_character(before) && !is_word_character(after) {
            return true;
        }
        // Like `indexOf(needle, index + 1)`: the next search starts one character later.
        from = index + haystack[index..].chars().next().map_or(1, char::len_utf8);
    }
    false
}

/// Kotlin's `isLetterOrDigit`, which sees a UTF-16 surrogate (half of a character beyond U+FFFF) as neither.
fn is_word_character(character: Option<char>) -> bool {
    character.is_some_and(|character| character.len_utf16() == 1 && character.is_alphanumeric())
}
