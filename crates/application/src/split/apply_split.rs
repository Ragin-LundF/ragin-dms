use std::sync::Arc;

use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::DocumentId;
use dms_domain::split::{SplitPart, SplitProposal};

use crate::common::OperationOutcome;
use crate::library::CatalogStore;
use crate::pdf::SplitDocument;
use crate::port::pdf::PdfEditor;
use crate::port::state::TaskStore;
use crate::split::{ApplySplitInput, ApplySplitOutcome};

/// Splits a stack at the cuts the user confirmed, with the existing split, then puts every part the AI said
/// something about up for review with those values as a suggestion; nothing of it is applied.
pub struct ApplySplit {
    store: Arc<CatalogStore>,
    pdf: Arc<dyn PdfEditor>,
    split_document: Arc<SplitDocument>,
    tasks: Arc<dyn TaskStore>,
}

impl ApplySplit {
    pub fn new(
        store: Arc<CatalogStore>,
        pdf: Arc<dyn PdfEditor>,
        split_document: Arc<SplitDocument>,
        tasks: Arc<dyn TaskStore>,
    ) -> Self {
        Self { store, pdf, split_document, tasks }
    }

    pub fn run(&self, input: &ApplySplitInput) -> ApplySplitOutcome {
        if !input.confirmed {
            return ApplySplitOutcome::NotSplit(OperationOutcome::NotConfirmed);
        }
        let Some(library) = self.store.current() else {
            return ApplySplitOutcome::NotSplit(OperationOutcome::LibraryNotOpen);
        };
        let Some(document) = library.catalog.documents().get(&input.document_id) else {
            return ApplySplitOutcome::NotSplit(OperationOutcome::NotFound);
        };
        let page_count = match self.pdf.page_count(library.folders.primary(), &document.path) {
            Ok(count) => count,
            Err(error) => return ApplySplitOutcome::NotSplit(OperationOutcome::StorageFailed(error)),
        };
        if let Some(problem) = SplitProposal::problem_of(&input.parts, page_count) {
            return ApplySplitOutcome::InvalidParts(problem);
        }
        if input.parts.len() < 2 {
            return ApplySplitOutcome::NothingToSplit;
        }
        let ranges: Vec<_> = input.parts.iter().map(|part| part.first_page..=part.last_page).collect();
        let (outcome, parts) = self.split_document.split(&input.document_id, &ranges, input.trash_original);
        if parts.is_empty() {
            return ApplySplitOutcome::NotSplit(outcome);
        }
        let without_suggestion = parts
            .iter()
            .zip(&input.parts)
            .filter(|(id, part)| !self.suggested(id, part, &input.model))
            .map(|(id, _)| id.clone())
            .collect();
        ApplySplitOutcome::Split { outcome, parts, without_suggestion }
    }

    /// `true` once the part's suggestion waits for review; a part without values or whose suggestion could not be
    /// saved has none.
    fn suggested(&self, document_id: &DocumentId, part: &SplitPart, model: &str) -> bool {
        if !part.has_suggestion() {
            return false;
        }
        let suggestion = ClassificationSuggestion {
            title: part.title.clone(),
            category: part.category.clone(),
            correspondent: part.correspondent.clone(),
            document_date: part.document_date,
            confidence: part.confidence.filter(|confidence| (0.0..=1.0).contains(confidence)),
            ..ClassificationSuggestion::empty(document_id.clone(), model)
        };
        suggestion.validate().is_ok() && self.tasks.save_suggestion(&suggestion).is_ok()
    }
}
