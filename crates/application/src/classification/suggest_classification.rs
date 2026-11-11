use std::sync::Arc;

use dms_domain::catalog::Catalog;
use dms_domain::classification::{ClassificationSuggestion, SuggestedCategory, SuggestedFolder, SuggestedTag};
use dms_domain::document::{Document, DocumentId};
use dms_domain::outline::FolderOutline;
use dms_domain::rule::{RuleMatcher, RuleResult};
use dms_domain::text;
use indexmap::IndexSet;

use crate::classification::{RuleSuggestions, SuggestOutcome};
use crate::common::LibrarySnapshot;
use crate::library::CatalogStore;
use crate::port::ai::{
    ClassificationRequest, ClassificationResult, DocumentClassifier, DocumentContent, DocumentContentReader,
};
use crate::port::state::TaskStore;
use crate::port::storage::{FolderOperation, TextStore};
use crate::port::system::Clock;
use crate::sync::Replicator;

const FILE_NAME_EXAMPLES: usize = 20;

/// Lets the AI suggest a classification for one document. The matching rules come first: when they decide
/// category and correspondent, the suggestion is made without the AI; otherwise the AI gets their values as fixed
/// and they override its answer. The suggestion is stored as a task for the user's review; nothing in the catalog
/// or on disk changes, except the document's text in `.dms-text/`, which search uses.
pub struct SuggestClassification {
    store: Arc<CatalogStore>,
    reader: Arc<dyn DocumentContentReader>,
    classifier: Arc<dyn DocumentClassifier>,
    tasks: Arc<dyn TaskStore>,
    texts: Arc<dyn TextStore>,
    replicator: Arc<Replicator>,
    clock: Arc<dyn Clock>,
}

impl SuggestClassification {
    pub fn new(
        store: Arc<CatalogStore>,
        reader: Arc<dyn DocumentContentReader>,
        classifier: Arc<dyn DocumentClassifier>,
        tasks: Arc<dyn TaskStore>,
        texts: Arc<dyn TextStore>,
        replicator: Arc<Replicator>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { store, reader, classifier, tasks, texts, replicator, clock }
    }

    pub fn run(&self, document_id: &DocumentId) -> SuggestOutcome {
        let Some(library) = self.store.current() else {
            return SuggestOutcome::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(document_id) else {
            return SuggestOutcome::NotFound;
        };
        let content = document.kind().readable_by_ai().then(|| self.reader.read(library.folders.primary(), document));
        let rules = self.rules_for(&library, document, content.as_ref());
        if rules.is_sufficient() {
            // The rules decided category and correspondent; the AI is not needed.
            let suggestion = RuleSuggestions::of(document, &rules, &library.catalog);
            return self.suggested(&library, content.as_ref(), suggestion);
        }
        let content = match content {
            None => return SuggestOutcome::NotReadable { detail: None },
            Some(DocumentContent::NotReadable(reason)) => return SuggestOutcome::NotReadable { detail: Some(reason) },
            Some(content) => content,
        };
        let mut request = self.request(&library.catalog, document, content.clone());
        request.fixed_values = (!rules.is_empty()).then(|| rules.clone());
        match self.classifier.suggest(&request) {
            ClassificationResult::Failed { reason, detail } => SuggestOutcome::Failed { reason, detail },
            ClassificationResult::Suggested(suggestion) => {
                let suggestion = normalized(suggestion, &request, &library.catalog, document);
                let suggestion = RuleSuggestions::over(suggestion, &rules, &library.catalog);
                self.suggested(&library, Some(&content), suggestion)
            }
        }
    }

    fn suggested(
        &self,
        library: &LibrarySnapshot,
        content: Option<&DocumentContent>,
        suggestion: ClassificationSuggestion,
    ) -> SuggestOutcome {
        // Kotlin threw when the task store could not be written; the suggestion is shown either way.
        let _ = self.tasks.save_suggestion(&suggestion);
        self.store_text(library, &suggestion.document_id, content, &suggestion);
        SuggestOutcome::Suggested(suggestion)
    }

    /// The rules on the document's text as far as it is known: its text layer or transcription, name and title.
    fn rules_for(
        &self,
        library: &LibrarySnapshot,
        document: &Document,
        content: Option<&DocumentContent>,
    ) -> RuleResult {
        let settings = library.catalog.settings();
        if settings.rules.is_empty() && settings.correspondents.is_empty() {
            return RuleResult { correspondent: document.metadata.correspondent.clone(), ..RuleResult::default() };
        }
        let text = match content {
            Some(DocumentContent::Text(text)) => Some(text.clone()),
            _ => self.texts.read(library.folders.primary(), &document.id),
        };
        let haystack = [text.as_deref(), Some(document.path.file_name()), document.metadata.title.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n");
        RuleMatcher::apply(&settings.rules, &settings.correspondents, document, &haystack)
    }

    fn request(&self, catalog: &Catalog, document: &Document, content: DocumentContent) -> ClassificationRequest {
        let others: Vec<&Document> = catalog.documents().values().filter(|it| it.id != document.id).collect();
        let inbox = &catalog.settings().layout.inbox;
        let outline = (inbox.contains(&document.path) && inbox.ai_target_folder)
            .then(|| FolderOutline::of(catalog, vec![inbox.path.clone()], FolderOutline::DEFAULT_LIMIT));
        let mut categories: Vec<_> = catalog.categories().values().cloned().collect();
        categories.sort_by(|first, second| text::compare(&first.name, &second.name));
        let mut folders: Vec<_> = others
            .iter()
            .filter_map(|it| it.path.parent())
            .filter(|parent| !parent.is_root())
            .collect::<IndexSet<_>>()
            .into_iter()
            .collect();
        folders.sort_by(|first, second| text::compare(first.value(), second.value()));
        let mut newest_first = others.clone();
        newest_first.sort_by_key(|it| std::cmp::Reverse(it.added_at));
        ClassificationRequest {
            document_id: document.id.clone(),
            current_file_name: document.path.file_name().to_owned(),
            content,
            categories,
            tags: catalog.tags().clone(),
            folders,
            file_name_examples: newest_first
                .iter()
                .take(FILE_NAME_EXAMPLES)
                .map(|it| it.path.file_name().to_owned())
                .collect(),
            today: self.clock.today(),
            outline,
            field_definitions: catalog.settings().field_definitions.clone(),
            default_currency: catalog.settings().layout.default_currency.clone(),
            fixed_values: None,
        }
    }

    /// The text is derived data: a failure to store it does not fail the suggestion.
    fn store_text(
        &self,
        library: &LibrarySnapshot,
        document_id: &DocumentId,
        content: Option<&DocumentContent>,
        suggestion: &ClassificationSuggestion,
    ) {
        let text = match content {
            Some(DocumentContent::Text(text)) => Some(text.as_str()),
            _ => suggestion.transcription.as_deref(),
        };
        let Some(text) = text.filter(|text| !text::is_blank(text)) else {
            return;
        };
        let _ = self.replicator.replicate(
            &library.folders,
            |primary| self.texts.write(primary, document_id, text),
            &FolderOperation::CopyText(document_id.clone()),
        );
    }
}

/// Ties the suggestion to the document and checks it against the catalog as it is now. A target folder is only
/// kept for inbox documents and only where the outline allows it.
fn normalized(
    suggestion: ClassificationSuggestion,
    request: &ClassificationRequest,
    catalog: &Catalog,
    document: &Document,
) -> ClassificationSuggestion {
    let known_category = match &suggestion.category {
        Some(SuggestedCategory::Existing(category_id)) => catalog.categories().contains_key(category_id),
        _ => true,
    };
    let target = suggestion.target_folder.as_ref().and_then(|folder| {
        let outline = request.outline.as_ref()?;
        if outline.contains(&folder.path) {
            Some(SuggestedFolder { is_new: false, ..folder.clone() })
        } else if outline.can_create(&folder.path) {
            Some(SuggestedFolder { is_new: true, ..folder.clone() })
        } else {
            None
        }
    });
    let tags = suggestion
        .tags
        .iter()
        .map(|tag| SuggestedTag { name: tag.name.clone(), is_new: !catalog.tags().contains(&tag.name) })
        .collect();
    let fields = suggestion
        .fields
        .iter()
        .filter(|(key, value)| {
            catalog.settings().field(key).map(|definition| definition.field_type) == Some(value.field_type())
        })
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    ClassificationSuggestion {
        document_id: document.id.clone(),
        category: if known_category { suggestion.category.clone() } else { None },
        tags,
        target_reason: if target.is_some() { suggestion.target_reason.clone() } else { None },
        target_folder: target,
        fields,
        ..suggestion
    }
}
