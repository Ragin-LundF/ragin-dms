use std::sync::Arc;

use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use dms_domain::document::DocumentId;
use dms_domain::taxonomy::{Category, CategoryId, TagName};
use indexmap::IndexSet;

use crate::catalog::CatalogEdit;
use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::system::IdGenerator;

/// Applies one manual change to metadata, categories or tags and saves the catalog to every library folder.
/// Manual categorization works without any AI.
pub struct EditCatalog {
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
    ids: Arc<dyn IdGenerator>,
}

impl EditCatalog {
    pub fn new(store: Arc<CatalogStore>, activity: Arc<ActivityRecorder>, ids: Arc<dyn IdGenerator>) -> Self {
        Self { store, activity, ids }
    }

    pub fn run(&self, edit: &CatalogEdit) -> OperationOutcome {
        let outcome = self.store.commit(|catalog| self.apply(catalog, edit));
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::CatalogEdited, edit.name());
        }
        outcome
    }

    fn apply(&self, catalog: &Catalog, edit: &CatalogEdit) -> Result<CatalogUpdate, DomainError> {
        match edit {
            CatalogEdit::UpdateMetadata { document_id, metadata } => {
                catalog.update_metadata(document_id, metadata.clone())
            }
            CatalogEdit::AssignCategory { document_id, category_id } => {
                Ok(catalog.assign_category(document_id, category_id.clone()))
            }
            CatalogEdit::AddTags { document_id, tags } => {
                Ok(change_tags(catalog, document_id, |current| current.iter().chain(tags).cloned().collect()))
            }
            CatalogEdit::RemoveTags { document_id, tags } => Ok(change_tags(catalog, document_id, |current| {
                current.iter().filter(|tag| !tags.contains(*tag)).cloned().collect()
            })),
            CatalogEdit::AddCategory { name, parent_id } => {
                let id = CategoryId::new(self.ids.new_id()).expect("invariant: generated ids are not blank");
                catalog.add_category(Category::new(id, name.clone(), parent_id.clone(), None)?)
            }
            CatalogEdit::RenameCategory { category_id, name } => catalog.rename_category(category_id, name),
            CatalogEdit::MoveCategory { category_id, parent_id } => {
                Ok(catalog.move_category(category_id, parent_id.as_ref()))
            }
            CatalogEdit::RemoveCategory { category_id } => Ok(catalog.remove_category(category_id)),
            CatalogEdit::RenameTag { from, to } => Ok(catalog.rename_tag(from, to.clone())),
            CatalogEdit::RemoveTag { tag } => Ok(catalog.remove_tag(tag)),
        }
    }
}

fn change_tags(
    catalog: &Catalog,
    document_id: &DocumentId,
    change: impl FnOnce(&IndexSet<TagName>) -> IndexSet<TagName>,
) -> CatalogUpdate {
    let Some(document) = catalog.documents().get(document_id) else {
        return CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound);
    };
    catalog.set_tags(document_id, &change(&document.tags))
}
