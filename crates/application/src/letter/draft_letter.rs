use std::sync::Arc;

use chrono::NaiveDate;
use dms_domain::document::Document;
use dms_domain::library::RelativePath;

use crate::common::source_texts;
use crate::letter::{DraftLetterInput, DraftLetterOutcome, DraftSource, detect_language, template_draft};
use crate::library::CatalogStore;
use crate::port::ai::DocumentContentReader;
use crate::port::assistant::{LetterAnswer, LetterDraft, LetterParty, LetterRequest, LetterWriter};
use crate::port::state::SettingsRepository;
use crate::port::storage::TextStore;

/// Drafts a letter: by the AI when it is switched on, otherwise, or when the AI fails, from the fixed template, so
/// letters work without AI. Only reads; the draft is shown for editing.
pub struct DraftLetter {
    store: Arc<CatalogStore>,
    reader: Arc<dyn DocumentContentReader>,
    texts: Arc<dyn TextStore>,
    settings: Arc<dyn SettingsRepository>,
    writer: Arc<dyn LetterWriter>,
}

impl DraftLetter {
    pub fn new(
        store: Arc<CatalogStore>,
        reader: Arc<dyn DocumentContentReader>,
        texts: Arc<dyn TextStore>,
        settings: Arc<dyn SettingsRepository>,
        writer: Arc<dyn LetterWriter>,
    ) -> Self {
        Self { store, reader, texts, settings, writer }
    }

    pub fn run(&self, input: &DraftLetterInput) -> DraftLetterOutcome {
        let Some(library) = self.store.current() else {
            return DraftLetterOutcome::LibraryNotOpen;
        };
        let Some(mut sources) =
            input.document_ids.iter().map(|id| library.catalog.documents().get(id)).collect::<Option<Vec<&Document>>>()
        else {
            return DraftLetterOutcome::NotFound;
        };
        sources.sort_by_key(|document| std::cmp::Reverse(day_of(document)));
        // Without readable preferences the AI counts as off, as on a new device.
        let ai = self.settings.load_preferences().map(|preferences| preferences.ai).unwrap_or_default();
        let budget = usize::try_from(ai.input_characters).unwrap_or_default();
        let source_texts =
            source_texts(self.texts.as_ref(), self.reader.as_ref(), library.folders.primary(), &sources, budget)
                .into_iter()
                .map(|(_, text)| text)
                .collect::<Vec<String>>();
        let newest = sources.first();
        let name =
            input.recipient.clone().or_else(|| newest.and_then(|document| document.metadata.correspondent.clone()));
        let request = LetterRequest {
            kind: input.kind,
            sender: input.sender.clone(),
            recipient: LetterParty { name: name.unwrap_or_default(), address_lines: Vec::new() },
            subject_facts: input.subject_facts.clone(),
            language: detect_language(&source_texts),
            source_texts,
        };
        let (draft, source) = self.drafted(&request, ai.enabled);
        DraftLetterOutcome::Drafted {
            draft,
            recipient: request.recipient,
            language: request.language,
            folder: newest.and_then(|document| document.path.parent()).unwrap_or_else(RelativePath::root),
            source,
        }
    }

    fn drafted(&self, request: &LetterRequest, ai_enabled: bool) -> (LetterDraft, DraftSource) {
        if !ai_enabled {
            return (template_draft(request), DraftSource::Template { ai_failure: None });
        }
        // The owner's name and address stay on this computer: the AI writes the body, the name is signed here.
        let anonymous = LetterRequest { sender: None, ..request.clone() };
        match self.writer.draft(&anonymous) {
            LetterAnswer::Drafted { draft, model } => {
                (signed(draft, request.sender.as_ref()), DraftSource::Ai { model })
            }
            LetterAnswer::Failed { reason, .. } => {
                (template_draft(request), DraftSource::Template { ai_failure: Some(reason) })
            }
        }
    }
}

fn signed(draft: LetterDraft, sender: Option<&LetterParty>) -> LetterDraft {
    let Some(name) = sender.map(|sender| sender.name.trim()).filter(|name| !name.is_empty()) else {
        return draft;
    };
    let body = format!("{}\n{name}", draft.body.trim_end());
    LetterDraft { body, ..draft }
}

fn day_of(document: &Document) -> NaiveDate {
    document.metadata.document_date.unwrap_or_else(|| document.added_at.date_naive())
}
