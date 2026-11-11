//! The Kotlin enum constant names the files store, per Rust enum.

use dms_application::port::state::{ActivityKind, AiProvider};
use dms_domain::document::ClassificationSource;
use dms_domain::field::{FieldRole, FieldType};
use dms_domain::library::FolderRole;
use dms_domain::query::FieldComparison;
use dms_domain::rule::RuleOrigin;

use crate::yaml::YamlError;

pub type Names<T> = [(T, &'static str)];

pub const CLASSIFICATION_SOURCES: &Names<ClassificationSource> =
    &[(ClassificationSource::Ai, "AI"), (ClassificationSource::Manual, "MANUAL"), (ClassificationSource::Rule, "RULE")];

pub const FIELD_TYPES: &Names<FieldType> = &[
    (FieldType::Text, "TEXT"),
    (FieldType::Number, "NUMBER"),
    (FieldType::Money, "MONEY"),
    (FieldType::Date, "DATE"),
    (FieldType::Boolean, "BOOLEAN"),
];

pub const FIELD_ROLES: &Names<FieldRole> = &[
    (FieldRole::DueDate, "DUE_DATE"),
    (FieldRole::CancelBy, "CANCEL_BY"),
    (FieldRole::ContractEnd, "CONTRACT_END"),
    (FieldRole::WarrantyEnd, "WARRANTY_END"),
];

pub const FIELD_COMPARISONS: &Names<FieldComparison> = &[
    (FieldComparison::Present, "PRESENT"),
    (FieldComparison::Absent, "ABSENT"),
    (FieldComparison::Equals, "EQUALS"),
    (FieldComparison::Contains, "CONTAINS"),
    (FieldComparison::LessThan, "LESS_THAN"),
    (FieldComparison::GreaterThan, "GREATER_THAN"),
    (FieldComparison::WithinDays, "WITHIN_DAYS"),
];

pub const RULE_ORIGINS: &Names<RuleOrigin> = &[(RuleOrigin::Learned, "LEARNED"), (RuleOrigin::Manual, "MANUAL")];

pub const FOLDER_ROLES: &Names<FolderRole> = &[(FolderRole::Primary, "PRIMARY"), (FolderRole::Sync, "SYNC")];

pub const AI_PROVIDERS: &Names<AiProvider> = &[(AiProvider::Local, "LOCAL"), (AiProvider::Anthropic, "ANTHROPIC")];

pub const ACTIVITY_KINDS: &Names<ActivityKind> = &[
    (ActivityKind::DocumentAdded, "DOCUMENT_ADDED"),
    (ActivityKind::DocumentMoved, "DOCUMENT_MOVED"),
    (ActivityKind::DocumentFiled, "DOCUMENT_FILED"),
    (ActivityKind::DocumentTrashed, "DOCUMENT_TRASHED"),
    (ActivityKind::DocumentRestored, "DOCUMENT_RESTORED"),
    (ActivityKind::FolderCreated, "FOLDER_CREATED"),
    (ActivityKind::FolderMoved, "FOLDER_MOVED"),
    (ActivityKind::FolderTrashed, "FOLDER_TRASHED"),
    (ActivityKind::CatalogEdited, "CATALOG_EDITED"),
    (ActivityKind::SuggestionApproved, "SUGGESTION_APPROVED"),
    (ActivityKind::ConflictResolved, "CONFLICT_RESOLVED"),
    (ActivityKind::Synchronized, "SYNCHRONIZED"),
    (ActivityKind::Restructured, "RESTRUCTURED"),
    (ActivityKind::Exported, "EXPORTED"),
    (ActivityKind::PdfEdited, "PDF_EDITED"),
];

/// `name` of the Kotlin enum constant.
pub fn name_of<T: PartialEq + Copy>(names: &Names<T>, value: T) -> &'static str {
    names.iter().find(|(candidate, _)| *candidate == value).map_or("", |(_, name)| name)
}

/// The constant called `name`, or `None`.
pub fn value_of<T: Copy>(names: &Names<T>, name: &str) -> Option<T> {
    names.iter().find(|(_, candidate)| *candidate == name).map(|(value, _)| *value)
}

/// `valueOf`, which failed for an unknown name and so made the file unreadable.
///
/// # Errors
/// Fails for a name that is no constant.
pub fn parse<T: Copy>(names: &Names<T>, name: &str) -> Result<T, YamlError> {
    value_of(names, name).ok_or_else(|| YamlError(format!("No enum constant {name}")))
}
