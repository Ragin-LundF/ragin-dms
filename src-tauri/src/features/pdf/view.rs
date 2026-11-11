//! Presenters of the PDF tools (the deriving part of Kotlin's `PdfActions` and `PageEditorController`).

use std::collections::HashSet;
use std::ops::RangeInclusive;

use dms_application::pdf::SearchableOutcome;
use dms_application::port::pdf::PageChange;
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentKind};
use dms_domain::text;

use super::dto::{MergeCandidateDto, MergeStartDto, PageItemDto};
use crate::messages;
use crate::session::UiMessage;

const MATCHES: usize = 30;
const FULL_TURN: i32 = 360;

pub fn merge_start(document: &Document) -> MergeStartDto {
    let file_name = document.path.file_name();
    let base = file_name.rfind('.').map_or(file_name, |dot| &file_name[..dot]);
    MergeStartDto { first: candidate_of(document), name: format!("{base} (merged)") }
}

/// Other PDFs whose path or title contains `query` (ignoring case), newest added first, at most 30.
pub fn merge_matches(catalog: &Catalog, query: &str, chosen: &[String]) -> Vec<MergeCandidateDto> {
    let wanted = text::lowercase(text::trim(query));
    let chosen: HashSet<&str> = chosen.iter().map(String::as_str).collect();
    let mut matches: Vec<&Document> = catalog
        .documents()
        .values()
        .filter(|document| document.kind() == DocumentKind::Pdf && !chosen.contains(document.id.value()))
        .filter(|document| wanted.is_empty() || searchable_text(document).contains(&wanted))
        .collect();
    matches.sort_by_key(|document| std::cmp::Reverse(document.added_at));
    matches.into_iter().take(MATCHES).map(candidate_of).collect()
}

/// What applying the page editor does: the new version's pages (`None` when nothing was turned, removed or moved)
/// and the page ranges of the new version to split into (one range: no split).
#[derive(Debug, PartialEq, Eq)]
pub struct PageEdit {
    pub changes: Option<Vec<PageChange>>,
    pub ranges: Vec<RangeInclusive<usize>>,
}

/// `None` when every page is removed.
pub fn page_edit(pages: &[PageItemDto]) -> Option<PageEdit> {
    let kept: Vec<&PageItemDto> = pages.iter().filter(|page| !page.deleted).collect();
    if kept.is_empty() {
        return None;
    }
    let reordered = pages.iter().enumerate().any(|(index, page)| usize::try_from(page.source).ok() != Some(index));
    let edited = reordered || pages.iter().any(|page| page.deleted || page.rotation.rem_euclid(FULL_TURN) != 0);
    let changes = edited.then(|| {
        kept.iter()
            .map(|page| PageChange {
                page: usize::try_from(page.source).unwrap_or(usize::MAX),
                rotation: page.rotation.rem_euclid(FULL_TURN),
            })
            .collect()
    });
    let mut ranges = Vec::new();
    let mut start = 0;
    let last = kept.len() - 1;
    for (index, page) in kept.iter().enumerate() {
        if page.split_after && index < last {
            ranges.push(start..=index);
            start = index + 1;
        }
    }
    ranges.push(start..=last);
    Some(PageEdit { changes, ranges })
}

pub fn searchable_message(outcome: &SearchableOutcome, name: &str) -> UiMessage {
    match outcome {
        SearchableOutcome::Made(outcome) => messages::of(outcome, &format!("{name} is searchable now")),
        SearchableOutcome::AlreadySearchable => UiMessage::info(format!("{name} has a text layer already")),
        SearchableOutcome::NoTranscription => {
            UiMessage::error(format!("{name} has no transcription yet: classify it with AI first, or use precise OCR"))
        }
        SearchableOutcome::OcrUnavailable => UiMessage::error("ocrmypdf is not installed on this device"),
        SearchableOutcome::NotPdf => UiMessage::error("Only PDF files can be made searchable"),
        SearchableOutcome::Failed(outcome) => messages::of(outcome, ""),
    }
}

fn candidate_of(document: &Document) -> MergeCandidateDto {
    MergeCandidateDto {
        document_id: document.id.value().to_owned(),
        name: document.metadata.title.clone().unwrap_or_else(|| document.path.file_name().to_owned()),
        path: document.path.value().to_owned(),
    }
}

fn searchable_text(document: &Document) -> String {
    let mut parts = vec![document.path.value()];
    parts.extend(document.metadata.title.as_deref());
    text::lowercase(&parts.join(" "))
}

#[cfg(test)]
mod tests;
