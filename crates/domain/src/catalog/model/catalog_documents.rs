use std::collections::HashSet;

use indexmap::{IndexMap, IndexSet};

use crate::DomainError;
use crate::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use crate::document::{ClassificationRecord, ContentHash, Document, DocumentId, DocumentMetadata};
use crate::error::require;
use crate::field::{FieldKey, FieldValue};
use crate::library::RelativePath;
use crate::retention::Retention;
use crate::taxonomy::{CategoryId, TagName};
use crate::text;

/// The changes to documents. Functions that take values a Kotlin `copy` would have checked return an error
/// where Kotlin threw `IllegalArgumentException`.
impl Catalog {
    /// # Errors
    /// Fails for an invalid document.
    pub fn add_document(&self, document: Document) -> Result<CatalogUpdate, DomainError> {
        document.validate()?;
        if self.documents.contains_key(&document.id) {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentAlreadyExists));
        }
        if self.find_by_path(&document.path).is_some() {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::PathAlreadyUsed));
        }
        if document.category_id.as_ref().is_some_and(|id| !self.categories.contains_key(id)) {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::CategoryNotFound));
        }
        let canonical = Document { tags: self.canonical_tags(&document.tags), ..document };
        Ok(self.applied(|catalog| {
            catalog.tags.extend(canonical.tags.iter().cloned());
            catalog.documents.insert(canonical.id.clone(), canonical);
        }))
    }

    /// Removes the catalog entry and its links; what happens to the file is the caller's decision.
    #[must_use]
    pub fn remove_document(&self, document_id: &DocumentId) -> CatalogUpdate {
        if !self.documents.contains_key(document_id) {
            return CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound);
        }
        self.applied(|catalog| {
            catalog.documents.shift_remove(document_id);
            catalog.links.retain(|link| !link.touches(document_id));
        })
    }

    /// Moves or renames one document.
    ///
    /// # Errors
    /// Fails for the root path.
    pub fn move_document(
        &self,
        document_id: &DocumentId,
        new_path: RelativePath,
    ) -> Result<CatalogUpdate, DomainError> {
        if self.find_by_path(&new_path).is_some_and(|owner| &owner.id != document_id) {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::PathAlreadyUsed));
        }
        if !self.documents.contains_key(document_id) {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound));
        }
        require(!new_path.is_root(), || "A document needs a file path".to_owned())?;
        Ok(self.update_document(document_id, IndexSet::new(), |document| document.path = new_path))
    }

    /// Moves many documents at once, in one revision: every document of `moves` gets its new path. Paths may be
    /// swapped among the moved documents; no path may end up used twice.
    ///
    /// # Errors
    /// Fails for the root path.
    pub fn move_documents(&self, moves: &IndexMap<DocumentId, RelativePath>) -> Result<CatalogUpdate, DomainError> {
        if moves.keys().any(|id| !self.documents.contains_key(id)) {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound));
        }
        require(!moves.values().any(RelativePath::is_root), || "A document needs a file path".to_owned())?;
        let mut relocated = self.documents.clone();
        for (id, path) in moves {
            if let Some(document) = relocated.get_mut(id) {
                document.path = path.clone();
            }
        }
        let paths: HashSet<&RelativePath> = relocated.values().map(|document| &document.path).collect();
        if paths.len() != relocated.len() {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::PathAlreadyUsed));
        }
        Ok(self.applied(|catalog| catalog.documents = relocated))
    }

    /// Moves or renames a folder: every document below `from` ends up below `to`.
    ///
    /// # Panics
    /// Never: every relocated path lies below `to` and keeps a valid remainder.
    #[must_use]
    pub fn move_folder(&self, from: &RelativePath, to: &RelativePath) -> CatalogUpdate {
        if from.is_root() || from == to || to.is_inside(from) {
            return CatalogUpdate::Rejected(CatalogRejection::InvalidFolderMove);
        }
        let (moving, staying): (Vec<&Document>, Vec<&Document>) =
            self.documents.values().partition(|document| document.path.is_inside(from));
        let moved: Vec<Document> = moving
            .into_iter()
            .map(|document| Document {
                path: document.path.relocate(from, to).expect("invariant: the document lies inside the moved folder"),
                ..document.clone()
            })
            .collect();
        let occupied: HashSet<&RelativePath> = staying.iter().map(|document| &document.path).collect();
        if moved.iter().any(|document| occupied.contains(&document.path)) {
            return CatalogUpdate::Rejected(CatalogRejection::PathAlreadyUsed);
        }
        self.applied(|catalog| {
            for document in moved {
                catalog.documents.insert(document.id.clone(), document);
            }
        })
    }

    /// Records new content of a file that was changed on disk.
    #[must_use]
    pub fn update_content(&self, document_id: &DocumentId, content_hash: ContentHash, size: u64) -> CatalogUpdate {
        self.update_document(document_id, IndexSet::new(), |document| {
            document.content_hash = content_hash;
            document.size = size;
        })
    }

    /// # Errors
    /// Fails for invalid metadata.
    pub fn update_metadata(
        &self,
        document_id: &DocumentId,
        metadata: DocumentMetadata,
    ) -> Result<CatalogUpdate, DomainError> {
        metadata.validate()?;
        Ok(self.update_document(document_id, IndexSet::new(), |document| document.metadata = metadata))
    }

    /// Sets or, with `None`, clears the category of a document.
    #[must_use]
    pub fn assign_category(&self, document_id: &DocumentId, category_id: Option<CategoryId>) -> CatalogUpdate {
        if category_id.as_ref().is_some_and(|id| !self.categories.contains_key(id)) {
            return CatalogUpdate::Rejected(CatalogRejection::CategoryNotFound);
        }
        self.update_document(document_id, IndexSet::new(), |document| document.category_id = category_id)
    }

    /// Replaces the tags of a document. Tags the catalog does not know yet are added to it.
    #[must_use]
    pub fn set_tags(&self, document_id: &DocumentId, new_tags: &IndexSet<TagName>) -> CatalogUpdate {
        let canonical = self.canonical_tags(new_tags);
        self.update_document(document_id, canonical.clone(), |document| document.tags = canonical)
    }

    /// # Errors
    /// Fails for an invalid record.
    pub fn record_classification(
        &self,
        document_id: &DocumentId,
        record: ClassificationRecord,
    ) -> Result<CatalogUpdate, DomainError> {
        record.validate()?;
        Ok(self.update_document(document_id, IndexSet::new(), |document| document.classification = Some(record)))
    }

    /// Replaces the custom field values of a document. Every key needs a definition in the settings, and every
    /// value the type of its definition. A value whose definition was removed may stay as it is, so the document
    /// keeps it until the user removes it.
    ///
    /// # Errors
    /// Fails for an invalid value, such as blank text.
    pub fn set_fields(
        &self,
        document_id: &DocumentId,
        fields: IndexMap<FieldKey, FieldValue>,
    ) -> Result<CatalogUpdate, DomainError> {
        fields.values().try_for_each(FieldValue::validate)?;
        let current = self.documents.get(document_id).map(|document| &document.fields);
        for (key, value) in &fields {
            match self.settings.field(key) {
                None if current.and_then(|current| current.get(key)) == Some(value) => {}
                None => return Ok(CatalogUpdate::Rejected(CatalogRejection::FieldNotDefined)),
                Some(definition) if definition.field_type != value.field_type() => {
                    return Ok(CatalogUpdate::Rejected(CatalogRejection::FieldTypeMismatch));
                }
                Some(_) => {}
            }
        }
        Ok(self.update_document(document_id, IndexSet::new(), |document| document.fields = fields))
    }

    /// Sets the note of a document; a blank note removes it.
    #[must_use]
    pub fn set_note(&self, document_id: &DocumentId, note: Option<&str>) -> CatalogUpdate {
        let cleaned = note.map(text::trim_end).filter(|note| !text::is_blank(note)).map(str::to_owned);
        self.update_document(document_id, IndexSet::new(), |document| document.note = cleaned)
    }

    /// Sets the opt-in retention of a document; `None` or an empty retention removes it.
    #[must_use]
    pub fn set_retention(&self, document_id: &DocumentId, retention: Option<Retention>) -> CatalogUpdate {
        let retention = retention.filter(|retention| !retention.is_empty());
        self.update_document(document_id, IndexSet::new(), |document| document.retention = retention)
    }

    fn update_document(
        &self,
        document_id: &DocumentId,
        added_tags: IndexSet<TagName>,
        change: impl FnOnce(&mut Document),
    ) -> CatalogUpdate {
        if !self.documents.contains_key(document_id) {
            return CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound);
        }
        self.applied(|catalog| {
            if let Some(document) = catalog.documents.get_mut(document_id) {
                change(document);
            }
            catalog.tags.extend(added_tags);
        })
    }

    /// Uses the spelling the catalog already knows for tags that differ only in casing.
    fn canonical_tags(&self, requested: &IndexSet<TagName>) -> IndexSet<TagName> {
        requested.iter().map(|tag| self.tags.get(tag).unwrap_or(tag).clone()).collect()
    }
}
