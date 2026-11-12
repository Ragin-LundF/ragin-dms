use chrono::{DateTime, Utc};
use indexmap::{IndexMap, IndexSet};

use crate::DomainError;
use crate::document::{ClassificationRecord, ContentHash, DocumentId, DocumentKind, DocumentMetadata};
use crate::error::require;
use crate::field::{FieldKey, FieldValue};
use crate::library::RelativePath;
use crate::retention::Retention;
use crate::tax::TaxAssignment;
use crate::taxonomy::{CategoryId, TagName};
use crate::text;

/// A file in the library together with everything the catalog knows about it: metadata, category, tags,
/// custom [`Self::fields`], a free-text [`Self::note`], an opt-in [`Self::retention`] and its place in a tax return
/// ([`Self::tax`]).
///
/// Fields are public like the properties of the Kotlin data class; [`Self::validate`] checks the rules its
/// `init` block checked, and [`crate::catalog::Catalog::new`] runs it for every document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub id: DocumentId,
    pub path: RelativePath,
    pub content_hash: ContentHash,
    pub size: u64,
    pub added_at: DateTime<Utc>,
    pub metadata: DocumentMetadata,
    pub category_id: Option<CategoryId>,
    pub tags: IndexSet<TagName>,
    pub classification: Option<ClassificationRecord>,
    pub fields: IndexMap<FieldKey, FieldValue>,
    pub note: Option<String>,
    pub retention: Option<Retention>,
    pub tax: Option<TaxAssignment>,
}

impl Document {
    /// A document with empty metadata, no category, tags, fields, note or retention.
    ///
    /// # Errors
    /// Fails for the root path.
    pub fn new(
        id: DocumentId,
        path: RelativePath,
        content_hash: ContentHash,
        size: u64,
        added_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let document = Self {
            id,
            path,
            content_hash,
            size,
            added_at,
            metadata: DocumentMetadata::default(),
            category_id: None,
            tags: IndexSet::new(),
            classification: None,
            fields: IndexMap::new(),
            note: None,
            retention: None,
            tax: None,
        };
        document.validate()?;
        Ok(document)
    }

    /// # Errors
    /// Fails for the root path, a blank note, an empty retention, or invalid metadata, classification or
    /// field values, or an invalid tax assignment.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(!self.path.is_root(), || "A document needs a file path".to_owned())?;
        require(self.note.as_deref().is_none_or(|note| !text::is_blank(note)), || {
            "Note must be null or not blank".to_owned()
        })?;
        require(self.retention.as_ref().is_none_or(|retention| !retention.is_empty()), || {
            "An empty retention is stored as none".to_owned()
        })?;
        self.metadata.validate()?;
        if let Some(classification) = &self.classification {
            classification.validate()?;
        }
        if let Some(tax) = &self.tax {
            tax.validate()?;
        }
        self.fields.values().try_for_each(FieldValue::validate)
    }

    #[must_use]
    pub fn kind(&self) -> DocumentKind {
        DocumentKind::of(&self.path)
    }
}
