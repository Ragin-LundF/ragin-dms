use chrono::NaiveDate;
use dms_domain::field::{FieldKey, FieldValue};
use dms_domain::library::RelativePath;
use dms_domain::taxonomy::TagName;
use indexmap::{IndexMap, IndexSet};

use crate::classification::ApprovedCategory;

/// The values of a suggestion the user approved, possibly edited. A `None` field was not approved and leaves the
/// document unchanged; `tags` are added to the document's tags. `target_folder` files the document there, together
/// with the approved `file_name`. `fields` are set on the document, next to the values it already has.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SuggestionApproval {
    pub title: Option<String>,
    pub document_date: Option<NaiveDate>,
    pub correspondent: Option<String>,
    pub category: Option<ApprovedCategory>,
    pub tags: IndexSet<TagName>,
    pub file_name: Option<String>,
    pub target_folder: Option<RelativePath>,
    pub fields: IndexMap<FieldKey, FieldValue>,
}
