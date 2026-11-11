//! Presenters of the Ask screen (the deriving part of Kotlin's `AskViewModel`).

use dms_application::ask::AskOutcome;
use dms_application::port::ai::ClassificationFailure;
use dms_domain::catalog::Catalog;
use dms_domain::query::DocumentQuery;

use super::dto::{AskEntryDto, CitationDto};
use crate::session::UiMessage;

const WHOLE_LIBRARY: &str = "Whole library";
const WORDS_ONLY: &str = "Searched by words only; update the index to search by meaning";

/// "Whole library", then "Smart folder: <name>" per saved search.
pub fn scope_labels(catalog: Option<&Catalog>) -> Vec<String> {
    let searches = catalog.map(|catalog| catalog.settings().saved_searches.as_slice()).unwrap_or_default();
    std::iter::once(WHOLE_LIBRARY.to_owned())
        .chain(searches.iter().map(|search| format!("Smart folder: {}", search.name)))
        .collect()
}

/// The query of the scope at `index` of [`scope_labels`]; the whole library for 0 or a scope that is gone.
pub fn scope_query(catalog: Option<&Catalog>, index: usize) -> DocumentQuery {
    index
        .checked_sub(1)
        .and_then(|search| catalog?.settings().saved_searches.get(search))
        .map(|search| search.query.clone())
        .unwrap_or_default()
}

pub fn asking(question: &str) -> AskEntryDto {
    AskEntryDto { asking: true, ..noted(question, None) }
}

/// A question that got no answer, with the reason.
pub fn noted(question: &str, note: Option<String>) -> AskEntryDto {
    AskEntryDto { question: question.to_owned(), answer: None, citations: Vec::new(), note, asking: false }
}

pub fn entry_of(question: &str, outcome: &AskOutcome) -> AskEntryDto {
    match outcome {
        AskOutcome::Answered { text, citations, used_vectors, .. } => AskEntryDto {
            question: question.to_owned(),
            answer: Some(text.clone()),
            citations: citations
                .iter()
                .map(|citation| CitationDto {
                    number: u32::try_from(citation.number).unwrap_or(u32::MAX),
                    document_id: citation.document_id.value().to_owned(),
                    title: citation.title.clone(),
                })
                .collect(),
            note: (!used_vectors).then(|| WORDS_ONLY.to_owned()),
            asking: false,
        },
        AskOutcome::NotFound => noted(question, Some("Nothing in your documents fits this question.".to_owned())),
        AskOutcome::LibraryNotOpen => noted(question, Some("No library is open.".to_owned())),
        AskOutcome::Failed { reason, .. } => noted(question, Some(failure_text(*reason).to_owned())),
    }
}

/// Kotlin `AiMessages.failure`.
// shortcut: duplicated here until the shell's messages.rs has the AI failure texts (noted in ui-needs.md).
pub fn failure_text(reason: ClassificationFailure) -> &'static str {
    match reason {
        ClassificationFailure::ProviderUnavailable => {
            "The AI does not answer. Start LM Studio's server, or for Anthropic set ANTHROPIC_API_KEY; check Settings › AI"
        }
        ClassificationFailure::ModelCannotReadImages => {
            "The model cannot read images. Load a vision model, such as Gemma 3 or Qwen2.5-VL"
        }
        ClassificationFailure::InvalidResponse => "The AI's answer could not be understood; try again",
        ClassificationFailure::Timeout => "The AI took too long; try again, or raise the timeout in Settings › AI",
    }
}

/// What "Update index" reports: how many documents were read anew, or why the index could not be built.
pub fn index_message(indexed: usize, failure: Option<&str>) -> UiMessage {
    match failure {
        None => UiMessage::info(format!("The index is up to date; {indexed} documents were read anew")),
        Some(reason) => UiMessage::error(format!("The index could not be built: {reason}")),
    }
}

#[cfg(test)]
mod tests;
