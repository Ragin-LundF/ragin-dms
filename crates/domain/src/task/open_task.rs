use chrono::NaiveDate;

use crate::catalog::CatalogMergeConflict;
use crate::change::{ContentConflict, DetectedChange, MissingFile, ModifiedFile, MovedFile, NewFile};
use crate::classification::ClassificationSuggestion;
use crate::dates;
use crate::document::{ContentHash, DocumentId};
use crate::field::FieldKey;
use crate::library::{LibraryFolderId, RelativePath};
use crate::rule::RuleProposal;
use crate::task::TaskKey;

/// Something on the dashboard that waits for the user's decision. Nothing changes the library until the user
/// acts on a task.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // a dashboard holds a few dozen tasks at most
pub enum OpenTask {
    /// Approve, edit or reject what the AI suggested for a document.
    ReviewSuggestion(ClassificationSuggestion),
    /// Add an uncatalogued file to the catalog (and let the AI suggest a classification).
    AdoptNewFile(NewFile),
    /// Accept new content of a file, or restore the catalogued content from another folder.
    ReviewModification(ModifiedFile),
    /// Apply a move made outside the tool to the catalog and the other folders, or undo it.
    ConfirmMove(MovedFile),
    /// Restore a missing file from a folder that has it, or delete it everywhere.
    RestoreOrDelete(MissingFile),
    /// Decide which content of a file to keep when the folders disagree.
    ResolveContentConflict(ContentConflict),
    /// Keep one of several catalogued documents with the same size and content hash, and move the other copies to
    /// the trash. `document_ids` has at least two entries.
    ResolveDuplicates { content_hash: ContentHash, document_ids: Vec<DocumentId> },
    /// Review a decision the catalog merge made between two catalog copies.
    ReviewCatalogMerge(CatalogMergeConflict),
    /// Retry operations that could not be applied to a folder, for example while it was offline.
    RetryFailedOperations { folder_id: LibraryFolderId, count: usize },
    /// A deadline field of a document (due date, cancel by, …) falls on `date`, within its lead time. The key holds
    /// the date, so a dismissed task returns when the date changes.
    DeadlineDue { document_id: DocumentId, field_key: FieldKey, date: NaiveDate },
    /// The retention the user set for a document ended on `date`: delete it, or mark it as done so it stays an
    /// ordinary document.
    RetentionDue { document_id: DocumentId, date: NaiveDate },
    /// `folder` holds `file_count` files without any structure; the restructure assistant could sort them.
    RestructureSuggested { folder: RelativePath, file_count: usize },
    /// Accept or reject a rule learned from the documents the user approved.
    ReviewLearnedRule(RuleProposal),
}

impl OpenTask {
    /// The task that asks the user what to do about `change`.
    #[must_use]
    pub fn for_change(change: DetectedChange) -> Self {
        match change {
            DetectedChange::NewFile(change) => Self::AdoptNewFile(change),
            DetectedChange::ModifiedFile(change) => Self::ReviewModification(change),
            DetectedChange::MovedFile(change) => Self::ConfirmMove(change),
            DetectedChange::MissingFile(change) => Self::RestoreOrDelete(change),
            DetectedChange::ContentConflict(change) => Self::ResolveContentConflict(change),
        }
    }

    #[must_use]
    pub fn key(&self) -> TaskKey {
        TaskKey::new(match self {
            Self::ReviewSuggestion(suggestion) => format!("suggestion:{}", suggestion.document_id),
            Self::AdoptNewFile(change) => format!("new:{}", change.path),
            Self::ReviewModification(change) => format!("modified:{}:{}", change.document_id, change.folder_id),
            Self::ConfirmMove(change) => format!("moved:{}:{}", change.document_id, change.folder_id),
            Self::RestoreOrDelete(change) => format!("missing:{}", change.document_id),
            Self::ResolveContentConflict(change) => format!("conflict:{}", change.path),
            Self::ResolveDuplicates { content_hash, .. } => format!("duplicates:{content_hash}"),
            Self::ReviewCatalogMerge(conflict) => format!("merge:{}", merge_subject(conflict)),
            Self::RetryFailedOperations { folder_id, .. } => format!("retry:{folder_id}"),
            Self::DeadlineDue { document_id, field_key, date } => {
                format!("deadline:{document_id}:{field_key}:{}", dates::iso(*date))
            }
            Self::RetentionDue { document_id, date } => format!("retention:{document_id}:{}", dates::iso(*date)),
            Self::RestructureSuggested { folder, file_count } => format!("restructure:{folder}:{file_count}"),
            Self::ReviewLearnedRule(proposal) => format!("rule:{}", proposal.signature()),
        })
    }
}

fn merge_subject(conflict: &CatalogMergeConflict) -> String {
    match conflict {
        CatalogMergeConflict::DocumentDiffers { kept, .. } => format!("document:{}", kept.id),
        CatalogMergeConflict::PathClash { discarded, .. } => format!("path:{}", discarded.id),
        CatalogMergeConflict::CategoryDiffers { kept, .. } => format!("category:{}", kept.id),
        CatalogMergeConflict::FieldDefinitionDiffers { kept, .. } => format!("field:{}", kept.key),
        CatalogMergeConflict::SavedSearchDiffers { kept, .. } => format!("search:{}", kept.id),
        CatalogMergeConflict::RuleDiffers { kept, .. } => format!("rule:{}", kept.id),
    }
}
