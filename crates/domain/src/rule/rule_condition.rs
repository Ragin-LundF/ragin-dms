use crate::DomainError;
use crate::error::require;
use crate::text;

/// One condition of a [`crate::rule::MatchingRule`]; every condition of a rule must hold.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RuleCondition {
    /// The document's correspondent, given or recognized in its text, is `name` (ignoring case).
    CorrespondentIs { name: String },
    /// The text contains all `words` with `require_all`, otherwise at least one of them (ignoring case).
    TextContains { words: Vec<String>, require_all: bool },
    /// The file name matches `glob`, where `*` stands for any characters and `?` for one (ignoring case).
    FileNameMatches { glob: String },
}

impl RuleCondition {
    /// # Errors
    /// Fails for a blank name, glob or word, or no words at all.
    pub fn validate(&self) -> Result<(), DomainError> {
        match self {
            Self::CorrespondentIs { name } => {
                require(!text::is_blank(name), || "Correspondent must not be blank".to_owned())
            }
            Self::TextContains { words, .. } => {
                require(!words.is_empty() && !words.iter().any(|word| text::is_blank(word)), || {
                    "Words must not be blank".to_owned()
                })
            }
            Self::FileNameMatches { glob } => require(!text::is_blank(glob), || "Glob must not be blank".to_owned()),
        }
    }
}
