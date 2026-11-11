use dms_domain::document::DocumentId;
use dms_domain::field::{FieldKey, FieldValue};
use dms_domain::retention::Retention;
use dms_domain::taxonomy::CategoryId;
use indexmap::IndexMap;

/// A change to a document's details that touches no file: custom fields, the note, retention and links.
#[derive(Debug, Clone, PartialEq)]
pub enum DetailEdit {
    /// Replaces the custom field values of a document.
    SetFields {
        document_id: DocumentId,
        fields: IndexMap<FieldKey, FieldValue>,
    },
    /// Sets the note; a blank note removes it.
    SetNote {
        document_id: DocumentId,
        note: Option<String>,
    },
    /// Sets the opt-in keep-until date of a document, or removes it with `None`.
    SetRetention {
        document_id: DocumentId,
        retention: Option<Retention>,
    },
    /// Sets the opt-in retention period of a category in years, or removes it with `None`.
    SetRetentionYears {
        category_id: CategoryId,
        years: Option<u32>,
    },
    Link {
        first: DocumentId,
        second: DocumentId,
        label: Option<String>,
    },
    Unlink {
        first: DocumentId,
        second: DocumentId,
    },
}

impl DetailEdit {
    /// The Kotlin class name, written to the activity log as the subject.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::SetFields { .. } => "SetFields",
            Self::SetNote { .. } => "SetNote",
            Self::SetRetention { .. } => "SetRetention",
            Self::SetRetentionYears { .. } => "SetRetentionYears",
            Self::Link { .. } => "Link",
            Self::Unlink { .. } => "Unlink",
        }
    }
}
