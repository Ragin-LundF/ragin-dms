use chrono::NaiveDate;
use indexmap::IndexMap;

use crate::DomainError;
use crate::classification::{SuggestedCategory, SuggestedFolder, SuggestedTag};
use crate::document::DocumentId;
use crate::error::require;
use crate::field::{FieldKey, FieldValue};
use crate::library::FileNamePolicy;
use crate::rule::RuleTarget;
use crate::text;

/// What the AI proposes for one document. A suggestion is a value, never applied state: nothing in it reaches
/// the catalog or the file name until the user approves it, field by field or as a whole.
///
/// [`Self::target_folder`] is where the document could be filed, with a one-line [`Self::target_reason`];
/// [`Self::fields`] are values for the library's custom fields; [`Self::rule_notes`] names the rule behind each
/// part that came from the matching rules instead of the AI.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassificationSuggestion {
    pub document_id: DocumentId,
    pub model: String,
    pub title: Option<String>,
    pub category: Option<SuggestedCategory>,
    pub tags: Vec<SuggestedTag>,
    pub document_date: Option<NaiveDate>,
    pub correspondent: Option<String>,
    pub file_name: Option<String>,
    pub transcription: Option<String>,
    pub confidence: Option<f64>,
    pub target_folder: Option<SuggestedFolder>,
    pub target_reason: Option<String>,
    pub fields: IndexMap<FieldKey, FieldValue>,
    pub rule_notes: IndexMap<RuleTarget, String>,
}

impl ClassificationSuggestion {
    /// The model name of suggestions the matching rules made without the AI.
    pub const RULES_MODEL: &'static str = "rules";

    /// A suggestion that proposes nothing yet; set the fields, then [`Self::validate`].
    #[must_use]
    pub fn empty(document_id: DocumentId, model: impl Into<String>) -> Self {
        Self {
            document_id,
            model: model.into(),
            title: None,
            category: None,
            tags: Vec::new(),
            document_date: None,
            correspondent: None,
            file_name: None,
            transcription: None,
            confidence: None,
            target_folder: None,
            target_reason: None,
            fields: IndexMap::new(),
            rule_notes: IndexMap::new(),
        }
    }

    /// # Errors
    /// Fails without a model name, for a file name the policy refuses, a confidence outside 0..=1, or an invalid
    /// part.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(!text::is_blank(&self.model), || "A suggestion must name its model".to_owned())?;
        require(self.file_name.as_deref().is_none_or(|name| FileNamePolicy::check(name).is_none()), || {
            format!(
                "Suggested file name violates the file name policy: {}",
                self.file_name.as_deref().unwrap_or_default()
            )
        })?;
        require(self.confidence.is_none_or(|confidence| (0.0..=1.0).contains(&confidence)), || {
            "Confidence must be between 0 and 1".to_owned()
        })?;
        if let Some(category) = &self.category {
            category.validate()?;
        }
        if let Some(folder) = &self.target_folder {
            folder.validate()?;
        }
        self.fields.values().try_for_each(FieldValue::validate)
    }

    /// `true` if the matching rules alone made this suggestion, without the AI.
    #[must_use]
    pub fn by_rules(&self) -> bool {
        self.model == Self::RULES_MODEL
    }
}
