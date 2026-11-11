use chrono::NaiveDate;
use dms_domain::document::DocumentId;
use dms_domain::field::{CurrencyCode, FieldDefinition};
use dms_domain::library::RelativePath;
use dms_domain::outline::FolderOutline;
use dms_domain::rule::RuleResult;
use dms_domain::taxonomy::{Category, TagName};
use indexmap::IndexSet;

use crate::port::ai::DocumentContent;

/// Everything the classifier gets for one document: its content and the library's existing taxonomy, folders and
/// file name examples, so it prefers what the library already uses. With an [`Self::outline`], the classifier also
/// suggests the folder to file the document in. It fills the library's custom [`Self::field_definitions`] where
/// the document has the values; a money value without a currency takes [`Self::default_currency`].
/// [`Self::fixed_values`] are what the user's matching rules decided; the classifier keeps them. Retention is never
/// part of a request: it is strictly the user's decision.
#[derive(Debug, Clone)]
pub struct ClassificationRequest {
    pub document_id: DocumentId,
    pub current_file_name: String,
    pub content: DocumentContent,
    pub categories: Vec<Category>,
    pub tags: IndexSet<TagName>,
    pub folders: Vec<RelativePath>,
    pub file_name_examples: Vec<String>,
    pub today: NaiveDate,
    pub outline: Option<FolderOutline>,
    pub field_definitions: Vec<FieldDefinition>,
    pub default_currency: Option<CurrencyCode>,
    pub fixed_values: Option<RuleResult>,
}
