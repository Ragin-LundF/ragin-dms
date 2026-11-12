use std::sync::Arc;

use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::{ClassificationRecord, ClassificationSource, Document, DocumentId, DocumentMetadata};
use dms_domain::library::RelativePath;
use dms_domain::taxonomy::{Category, CategoryId};
use dms_domain::text;
use indexmap::IndexSet;

use crate::classification::{ApprovedCategory, SuggestionApproval};
use crate::common::{ActivityRecorder, DocumentFiling, LibrarySnapshot, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::{ActivityKind, TaskStore};
use crate::port::system::{Clock, IdGenerator};

/// Applies the approved parts of a suggestion: renames the file in every folder if a new name was approved and
/// files it into the approved target folder, in one move, then writes metadata, category and tags to the catalog
/// in one change and records that the AI classification was confirmed. Standard fields the approved values need
/// (such as "IBAN") are defined on first use.
pub struct ApproveSuggestion {
    store: Arc<CatalogStore>,
    tasks: Arc<dyn TaskStore>,
    filing: Arc<DocumentFiling>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

/// Unwraps an applied update, or returns the rejection from the enclosing function.
macro_rules! applied {
    ($update:expr) => {
        match $update {
            CatalogUpdate::Applied(catalog) => catalog,
            rejected => return Ok(rejected),
        }
    };
}

impl ApproveSuggestion {
    pub fn new(
        store: Arc<CatalogStore>,
        tasks: Arc<dyn TaskStore>,
        filing: Arc<DocumentFiling>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { store, tasks, filing, activity, clock, ids }
    }

    pub fn run(&self, document_id: &DocumentId, approval: &SuggestionApproval) -> OperationOutcome {
        // Kotlin checked the category path when the approval was built.
        if let Some(Err(error)) = approval.category.as_ref().map(ApprovedCategory::validate) {
            return OperationOutcome::InvalidInput(error);
        }
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(document_id).cloned() else {
            return OperationOutcome::NotFound;
        };
        // A task store that cannot be read has no suggestion to approve.
        let suggestion =
            self.tasks.suggestions().unwrap_or_default().into_iter().find(|it| &it.document_id == document_id);
        let Some(suggestion) = suggestion else {
            return OperationOutcome::NotFound;
        };
        // The ids of the categories a path may create, fixed here so the check and the commit agree.
        let new_category_ids = self.new_category_ids_for(approval.category.as_ref());
        // Check the catalog part first, so a refusal does not leave a renamed file behind.
        match self.classify(&library.catalog, &suggestion, approval, &new_category_ids) {
            Err(error) => return OperationOutcome::InvalidInput(error),
            Ok(CatalogUpdate::Rejected(reason)) => return OperationOutcome::Rejected(reason),
            Ok(CatalogUpdate::Applied(_)) => {}
        }
        let renamed = self.move_file(&library, &document, approval);
        let OperationOutcome::Done { pending_folders, .. } = renamed else {
            return renamed;
        };
        let outcome = self
            .store
            .commit(|catalog| self.classify(catalog, &suggestion, approval, &new_category_ids))
            .with_pending_folders(&pending_folders);
        if let OperationOutcome::Done { catalog, .. } = &outcome {
            // Best effort: the classification is saved, a leftover suggestion is only shown again.
            let _ = self.tasks.remove_suggestion(document_id);
            let path = catalog.documents().get(document_id).map_or(&document.path, |approved| &approved.path);
            self.activity.record(ActivityKind::SuggestionApproved, path.value());
        }
        outcome
    }

    fn new_category_ids_for(&self, category: Option<&ApprovedCategory>) -> Vec<CategoryId> {
        let levels = match category {
            Some(ApprovedCategory::Path(names)) => names.len(),
            _ => 0,
        };
        (0..levels)
            .map(|_| CategoryId::new(self.ids.new_id()).expect("invariant: generated ids are not blank"))
            .collect()
    }

    fn move_file(
        &self,
        library: &LibrarySnapshot,
        document: &Document,
        approval: &SuggestionApproval,
    ) -> OperationOutcome {
        let parent = document.path.parent().unwrap_or_else(RelativePath::root);
        let folder = approval.target_folder.clone().unwrap_or_else(|| parent.clone());
        let file_name = approval.file_name.as_deref().unwrap_or_else(|| document.path.file_name());
        if folder == parent && file_name == document.path.file_name() {
            return OperationOutcome::Done { catalog: library.catalog.clone(), pending_folders: IndexSet::new() };
        }
        self.filing.file(&document.id, &folder, Some(file_name))
    }

    fn classify(
        &self,
        catalog: &Catalog,
        suggestion: &ClassificationSuggestion,
        approval: &SuggestionApproval,
        new_category_ids: &[CategoryId],
    ) -> Result<CatalogUpdate, DomainError> {
        let document_id = &suggestion.document_id;
        let Some(document) = catalog.documents().get(document_id) else {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound));
        };
        let metadata = DocumentMetadata {
            title: approval.title.clone().or_else(|| document.metadata.title.clone()),
            document_date: approval.document_date.or(document.metadata.document_date),
            correspondent: approval.correspondent.clone().or_else(|| document.metadata.correspondent.clone()),
        };
        let record = ClassificationRecord {
            source: if suggestion.by_ai() { ClassificationSource::Ai } else { ClassificationSource::Rule },
            model: Some(suggestion.model.clone()),
            confirmed_at: self.clock.now(),
        };
        let mut tags = document.tags.clone();
        tags.extend(approval.tags.iter().cloned());
        let mut fields = document.fields.clone();
        fields.extend(approval.fields.iter().map(|(key, value)| (key.clone(), value.clone())));

        let current = applied!(with_category(catalog, document_id, approval.category.as_ref(), new_category_ids)?);
        let current = applied!(current.update_metadata(document_id, metadata)?);
        let current = applied!(current.set_tags(document_id, &tags));
        let current = if approval.fields.is_empty() {
            current
        } else {
            let current = applied!(current.define_standard_fields(approval.fields.keys()));
            applied!(current.set_fields(document_id, fields)?)
        };
        let current = match &approval.tax {
            Some(tax) => applied!(current.set_tax(document_id, Some(tax.clone()))?),
            None => current,
        };
        current.record_classification(document_id, record)
    }
}

fn with_category(
    catalog: &Catalog,
    document_id: &DocumentId,
    category: Option<&ApprovedCategory>,
    new_ids: &[CategoryId],
) -> Result<CatalogUpdate, DomainError> {
    match category {
        None => Ok(CatalogUpdate::Applied(catalog.clone())),
        Some(ApprovedCategory::Existing(category_id)) => {
            Ok(catalog.assign_category(document_id, Some(category_id.clone())))
        }
        Some(ApprovedCategory::Path(names)) => with_path(catalog, document_id, names, new_ids),
    }
}

/// Walks `names` from the top level down: a level whose name already exists at that place, ignoring case, is
/// reused, every missing one is created, and the document gets the last level.
fn with_path(
    catalog: &Catalog,
    document_id: &DocumentId,
    names: &[String],
    new_ids: &[CategoryId],
) -> Result<CatalogUpdate, DomainError> {
    let mut current = catalog.clone();
    let mut parent_id: Option<CategoryId> = None;
    for (name, new_id) in names.iter().zip(new_ids) {
        if let Some(existing) = child_named(&current, parent_id.as_ref(), name) {
            parent_id = Some(existing.id.clone());
        } else {
            let created = Category {
                id: new_id.clone(),
                name: text::trim(name).to_owned(),
                parent_id: parent_id.clone(),
                retention_years: None,
            };
            current = applied!(current.add_category(created)?);
            parent_id = Some(new_id.clone());
        }
    }
    Ok(current.assign_category(document_id, Some(parent_id.expect("invariant: a path has at least one name"))))
}

fn child_named<'a>(catalog: &'a Catalog, parent_id: Option<&CategoryId>, name: &str) -> Option<&'a Category> {
    let key = text::lowercase(text::trim(name));
    catalog
        .categories()
        .values()
        .find(|category| category.parent_id.as_ref() == parent_id && text::lowercase(text::trim(&category.name)) == key)
}
