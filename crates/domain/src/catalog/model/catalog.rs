use std::collections::HashSet;

use indexmap::{IndexMap, IndexSet};

use crate::DomainError;
use crate::cases::{Case, CaseId};
use crate::catalog::{CatalogRecords, CatalogRevision, CatalogUpdate, LibraryId};
use crate::contract::{Contract, ContractId};
use crate::document::{ContentHash, Document, DocumentId};
use crate::error::require;
use crate::library::RelativePath;
use crate::link::DocumentLink;
use crate::settings::CatalogSettings;
use crate::tax::TaxYear;
use crate::taxonomy::{Category, CategoryId, TagName};

/// The aggregate root: every document of a library with its categorization, plus the categories and tags in
/// use, the [`Self::links`] between documents, the library-wide [`Self::settings`] and the [`Self::records`]
/// (contracts, cases, tax years).
///
/// A catalog is immutable. Every change goes through one of its functions, which returns either a new catalog
/// with the next [`Self::revision`] or the reason the change was refused. The invariants checked in
/// [`Self::new`] therefore hold for every catalog that exists.
///
/// The maps and sets keep insertion order, like Kotlin's `LinkedHashMap`/`LinkedHashSet`; their equality ignores
/// the order, as in Kotlin.
#[derive(Debug, Clone, PartialEq)]
pub struct Catalog {
    pub(super) library_id: LibraryId,
    pub(super) revision: CatalogRevision,
    pub(super) documents: IndexMap<DocumentId, Document>,
    pub(super) categories: IndexMap<CategoryId, Category>,
    pub(super) tags: IndexSet<TagName>,
    pub(super) links: IndexSet<DocumentLink>,
    pub(super) settings: CatalogSettings,
    pub(super) records: CatalogRecords,
}

impl Catalog {
    /// # Errors
    /// Fails when keys do not match ids, two documents share a path, a category, tag or link refers to
    /// something unknown, the category tree has a cycle, two links connect the same documents, or any part is
    /// invalid on its own. The catalog starts without records; see [`Self::with_records`].
    pub fn new(
        library_id: LibraryId,
        revision: CatalogRevision,
        documents: IndexMap<DocumentId, Document>,
        categories: IndexMap<CategoryId, Category>,
        tags: IndexSet<TagName>,
        links: IndexSet<DocumentLink>,
        settings: CatalogSettings,
    ) -> Result<Self, DomainError> {
        let records = CatalogRecords::default();
        let catalog = Self { library_id, revision, documents, categories, tags, links, settings, records };
        catalog.validate()?;
        Ok(catalog)
    }

    /// The same catalog, same revision, with `records`.
    ///
    /// # Errors
    /// Fails when a record is invalid or refers to an unknown document, or the emergency folder refers to an unknown
    /// contract.
    pub fn with_records(self, records: CatalogRecords) -> Result<Self, DomainError> {
        let catalog = Self { records, ..self };
        catalog.validate()?;
        catalog.validate_emergency_contracts()?;
        Ok(catalog)
    }

    #[must_use]
    pub fn empty(library_id: LibraryId) -> Self {
        Self {
            library_id,
            revision: CatalogRevision::INITIAL,
            documents: IndexMap::new(),
            categories: IndexMap::new(),
            tags: IndexSet::new(),
            links: IndexSet::new(),
            settings: CatalogSettings::default(),
            records: CatalogRecords::default(),
        }
    }

    fn validate(&self) -> Result<(), DomainError> {
        let documents = &self.documents;
        let categories = &self.categories;
        require(documents.iter().all(|(id, document)| id == &document.id), || {
            "Document keys must match ids".to_owned()
        })?;
        let paths: HashSet<&RelativePath> = documents.values().map(|document| &document.path).collect();
        require(paths.len() == documents.len(), || "Two documents share a path".to_owned())?;
        require(categories.iter().all(|(id, category)| id == &category.id), || {
            "Category keys must match ids".to_owned()
        })?;
        require(
            categories
                .values()
                .all(|category| category.parent_id.as_ref().is_none_or(|parent| categories.contains_key(parent))),
            || "A category refers to an unknown parent".to_owned(),
        )?;
        require(!categories.keys().any(|id| is_in_cycle(id, categories)), || {
            "The category tree contains a cycle".to_owned()
        })?;
        require(
            documents
                .values()
                .all(|document| document.category_id.as_ref().is_none_or(|id| categories.contains_key(id))),
            || "A document refers to an unknown category".to_owned(),
        )?;
        require(documents.values().all(|document| document.tags.is_subset(&self.tags)), || {
            "A document uses a tag the catalog does not know".to_owned()
        })?;
        require(
            self.links.iter().all(|link| documents.contains_key(link.first()) && documents.contains_key(link.second())),
            || "A link refers to an unknown document".to_owned(),
        )?;
        let pairs: HashSet<(&DocumentId, &DocumentId)> =
            self.links.iter().map(|link| (link.first(), link.second())).collect();
        require(pairs.len() == self.links.len(), || "Two links connect the same documents".to_owned())?;
        documents.values().try_for_each(Document::validate)?;
        categories.values().try_for_each(Category::validate)?;
        self.settings.validate()?;
        self.records.validate()?;
        self.validate_record_references()
    }

    /// Records and the emergency folder refer only to documents the catalog has.
    fn validate_record_references(&self) -> Result<(), DomainError> {
        let emergency =
            self.settings.emergency_folder.iter().flat_map(crate::emergency::EmergencyFolder::referenced_documents);
        require(self.records.referenced_documents().chain(emergency).all(|id| self.documents.contains_key(id)), || {
            "A record refers to an unknown document".to_owned()
        })
    }

    fn validate_emergency_contracts(&self) -> Result<(), DomainError> {
        let mut contracts =
            self.settings.emergency_folder.iter().flat_map(crate::emergency::EmergencyFolder::referenced_contracts);
        require(contracts.all(|id| self.records.contracts.contains_key(id)), || {
            "The emergency folder refers to an unknown contract".to_owned()
        })
    }

    #[must_use]
    pub fn library_id(&self) -> &LibraryId {
        &self.library_id
    }

    #[must_use]
    pub fn revision(&self) -> CatalogRevision {
        self.revision
    }

    #[must_use]
    pub fn documents(&self) -> &IndexMap<DocumentId, Document> {
        &self.documents
    }

    #[must_use]
    pub fn categories(&self) -> &IndexMap<CategoryId, Category> {
        &self.categories
    }

    #[must_use]
    pub fn tags(&self) -> &IndexSet<TagName> {
        &self.tags
    }

    #[must_use]
    pub fn links(&self) -> &IndexSet<DocumentLink> {
        &self.links
    }

    #[must_use]
    pub fn settings(&self) -> &CatalogSettings {
        &self.settings
    }

    #[must_use]
    pub fn records(&self) -> &CatalogRecords {
        &self.records
    }

    #[must_use]
    pub fn contracts(&self) -> &indexmap::IndexMap<ContractId, Contract> {
        &self.records.contracts
    }

    #[must_use]
    pub fn cases(&self) -> &indexmap::IndexMap<CaseId, Case> {
        &self.records.cases
    }

    #[must_use]
    pub fn tax_years(&self) -> &indexmap::IndexMap<i32, TaxYear> {
        &self.records.tax_years
    }

    // --- Queries

    #[must_use]
    pub fn find_by_path(&self, path: &RelativePath) -> Option<&Document> {
        self.documents.values().find(|document| &document.path == path)
    }

    #[must_use]
    pub fn find_by_content_hash(&self, content_hash: &ContentHash) -> Vec<&Document> {
        self.documents.values().filter(|document| &document.content_hash == content_hash).collect()
    }

    /// Documents directly or indirectly below `folder`.
    #[must_use]
    pub fn documents_inside(&self, folder: &RelativePath) -> Vec<&Document> {
        self.documents.values().filter(|document| document.path.is_inside(folder)).collect()
    }

    /// The links of `document_id`, whatever end it is on.
    #[must_use]
    pub fn links_of(&self, document_id: &DocumentId) -> Vec<&DocumentLink> {
        self.links.iter().filter(|link| link.touches(document_id)).collect()
    }

    // --- Links

    /// Links two documents; an existing link between them takes the new label.
    #[must_use]
    pub fn link(&self, link: DocumentLink) -> CatalogUpdate {
        if !self.documents.contains_key(link.first()) || !self.documents.contains_key(link.second()) {
            return CatalogUpdate::Rejected(crate::catalog::CatalogRejection::DocumentNotFound);
        }
        self.applied(|catalog| {
            catalog.links.retain(|existing| !existing.same_pair(&link));
            catalog.links.insert(link);
        })
    }

    #[must_use]
    pub fn unlink(&self, first: &DocumentId, second: &DocumentId) -> CatalogUpdate {
        let connects = |link: &DocumentLink| link.touches(first) && link.other_than(first) == second;
        if !self.links.iter().any(connects) {
            return CatalogUpdate::Rejected(crate::catalog::CatalogRejection::LinkNotFound);
        }
        self.applied(|catalog| catalog.links.retain(|link| !connects(link)))
    }

    // --- Settings

    /// Replaces the library-wide settings: field definitions, saved searches, rules, layout, owner, expectations,
    /// the emergency folder and the rest.
    ///
    /// # Errors
    /// Fails for invalid settings, or an emergency folder that refers to an unknown document or contract.
    pub fn change_settings(&self, settings: CatalogSettings) -> Result<CatalogUpdate, DomainError> {
        settings.validate()?;
        let changed = Self { settings, ..self.clone() };
        changed.validate_record_references()?;
        changed.validate_emergency_contracts()?;
        Ok(self.applied(|catalog| catalog.settings = changed.settings))
    }

    // --- Helpers

    /// A copy with `change` applied and the next revision. Callers keep the invariants themselves.
    pub(super) fn applied(&self, change: impl FnOnce(&mut Self)) -> CatalogUpdate {
        let mut catalog = self.clone();
        change(&mut catalog);
        catalog.revision = self.revision.next();
        CatalogUpdate::Applied(catalog)
    }

    /// The same catalog with other parts, for [`crate::catalog::CatalogMerger`]; no checks, no new revision.
    /// References to documents or contracts the parts lack are dropped.
    pub(crate) fn with_parts(
        &self,
        documents: IndexMap<DocumentId, Document>,
        categories: IndexMap<CategoryId, Category>,
        tags: IndexSet<TagName>,
        links: IndexSet<DocumentLink>,
        settings: CatalogSettings,
        records: CatalogRecords,
    ) -> Self {
        Self {
            library_id: self.library_id.clone(),
            revision: self.revision,
            documents,
            categories,
            tags,
            links,
            settings,
            records,
        }
        .without_dangling_references()
    }

    /// The catalog without references of records and the emergency folder to documents or contracts it lacks.
    fn without_dangling_references(mut self) -> Self {
        let documents = &self.documents;
        let records = self.records.retaining_documents(&|id| documents.contains_key(id));
        let emergency_folder =
            self.settings.emergency_folder.as_ref().map(|folder| {
                folder.retaining(&|id| documents.contains_key(id), &|id| records.contracts.contains_key(id))
            });
        self.records = records;
        self.settings.emergency_folder = emergency_folder;
        self
    }

    pub(crate) fn with_revision(mut self, revision: CatalogRevision) -> Self {
        self.revision = revision;
        self
    }
}

/// `true` if following the parents of `category_id` leads back to it.
pub(super) fn is_in_cycle(category_id: &CategoryId, parents: &IndexMap<CategoryId, Category>) -> bool {
    let mut visited = HashSet::new();
    let mut current = parents.get(category_id).and_then(|category| category.parent_id.as_ref());
    while let Some(parent) = current {
        if parent == category_id || !visited.insert(parent) {
            return true;
        }
        current = parents.get(parent).and_then(|category| category.parent_id.as_ref());
    }
    false
}
