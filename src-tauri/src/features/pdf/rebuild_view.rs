//! The words and the comparison of "Convert to real PDF" (Kotlin `RebuildTexts`, the deriving part of
//! `RebuildController`).

use dms_application::common::OperationOutcome;
use dms_application::pdf::rebuild::{RebuildBlocker, RebuildChoice, RebuildOutcome};
use dms_domain::rebuild::{KeepReason, LayoutBox, RebuiltPage, RegionKind};

use super::dto::{RebuildCompareDto, RebuiltPageDto, RegionDto, RegionKindDto};
use super::state::Comparison;
use crate::messages;
use crate::session::{OperationProgress, UiMessage};

pub const OPEN_COMPARISON: &str = "Choose what to keep in the open comparison first";

/// The message after a run; `None` for a cancelled one, which the session reports itself.
pub fn rebuild_message(outcome: &RebuildOutcome, name: &str) -> Option<UiMessage> {
    match outcome {
        RebuildOutcome::Rebuilt { rebuilt, .. } => Some(UiMessage::info(
            failed_pages(&rebuilt.pages).unwrap_or_else(|| format!("{name} is rebuilt: compare and choose")),
        )),
        RebuildOutcome::NotAvailable(blocker) => Some(UiMessage::error(blocker_text(*blocker, name))),
        RebuildOutcome::Failed(outcome) => Some(messages::of(outcome, "")),
        RebuildOutcome::Cancelled => None,
    }
}

pub fn progress_of(page: usize, pages: usize) -> OperationProgress {
    let step = if page < pages {
        format!("Page {} of {pages} · a few minutes per page", page + 1)
    } else {
        "Writing the PDF".to_owned()
    };
    OperationProgress::new(step, page, Some(pages))
}

pub fn present_compare(comparison: &Comparison) -> RebuildCompareDto {
    // Sizes stay exact in an f64 up to 8 PB.
    #[allow(clippy::cast_precision_loss)]
    let bytes = |size: u64| size as f64;
    RebuildCompareDto {
        document_id: comparison.document_id.value().to_owned(),
        rebuild_id: comparison.rebuilt.id.clone(),
        name: comparison.path.file_name().to_owned(),
        path: comparison.path.value().to_owned(),
        original_size: bytes(comparison.original_size),
        rebuilt_size: bytes(comparison.rebuilt.size),
        pages: comparison.rebuilt.pages.iter().map(present_page).collect(),
    }
}

pub fn choice_label(choice: RebuildChoice, name: &str) -> String {
    match choice {
        RebuildChoice::KeepOriginal => format!("Keeping the original of {name}"),
        RebuildChoice::ReplaceWithRebuild => format!("Replacing {name} with its rebuild"),
        RebuildChoice::KeepBoth => format!("Adding the rebuild of {name}"),
    }
}

pub fn choice_message(outcome: &OperationOutcome, choice: RebuildChoice, name: &str) -> UiMessage {
    if matches!(outcome, OperationOutcome::SourceChanged) {
        return UiMessage::error(format!("{name} changed after it was converted: convert it again, or keep both"));
    }
    let done = match choice {
        RebuildChoice::KeepOriginal => format!("Kept the original of {name}"),
        RebuildChoice::ReplaceWithRebuild => format!("{name} is a real PDF now; the scan is in the trash"),
        RebuildChoice::KeepBoth => format!("Added the rebuild of {name} next to it"),
    };
    messages::of(outcome, &done)
}

/// Whether the comparison closes after `outcome`: on success, or when the document or library is gone, since the
/// rebuild has nowhere to go then. Otherwise it stays, so the user can choose again.
pub fn closes_comparison(outcome: &OperationOutcome) -> bool {
    matches!(outcome, OperationOutcome::Done { .. } | OperationOutcome::NotFound | OperationOutcome::LibraryNotOpen)
}

fn present_page(page: &RebuiltPage) -> RebuiltPageDto {
    let count = |value: usize| u32::try_from(value).unwrap_or(u32::MAX);
    RebuiltPageDto {
        lines_as_text: count(page.lines_as_text),
        lines_total: count(page.lines_total),
        kept: kept_summary(page),
        regions: page
            .regions
            .iter()
            .map(|region| RegionDto {
                kind: match region.kind {
                    RegionKind::Text => RegionKindDto::Text,
                    RegionKind::KeptLine => RegionKindDto::KeptLine,
                    RegionKind::Image => RegionKindDto::Image,
                },
                left: scaled(region.bounds.left),
                top: scaled(region.bounds.top),
                right: scaled(region.bounds.right),
                bottom: scaled(region.bounds.bottom),
            })
            .collect(),
    }
}

/// A layout coordinate (0..=1000) as a whole unit; out-of-range values are clamped.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to 0..=1000 first
fn scaled(value: f64) -> u32 {
    value.clamp(0.0, LayoutBox::SCALE).round() as u32
}

/// "Kept as images: 3 × the AI was not sure of every character; …", most frequent reason first.
fn kept_summary(page: &RebuiltPage) -> Option<String> {
    let mut kept: Vec<(KeepReason, usize)> = page.kept_reasons.iter().map(|(reason, n)| (*reason, *n)).collect();
    if kept.is_empty() {
        return None;
    }
    kept.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    let parts: Vec<String> = kept.iter().map(|(reason, n)| format!("{n} × {}", reason_text(*reason))).collect();
    Some(format!("Kept as images: {}", parts.join("; ")))
}

fn reason_text(reason: KeepReason) -> &'static str {
    match reason {
        KeepReason::NotSure => "the AI was not sure of every character",
        KeepReason::NoInk => "no print where the AI saw the line",
        KeepReason::Covered => "covered by a stamp, signature or handwriting",
        KeepReason::NotEncodable => "a character the font does not have",
        KeepReason::TextMismatch => "the text does not match the print",
        KeepReason::LostCharacters => "the AI lost characters",
        KeepReason::PageFailed => "the AI could not read the page",
    }
}

/// "Page 3 stayed an image (…)", or `None` if every page was read.
fn failed_pages(pages: &[RebuiltPage]) -> Option<String> {
    let failed: Vec<String> = pages
        .iter()
        .enumerate()
        .filter(|(_, page)| page.kept_reasons.contains_key(&KeepReason::PageFailed))
        .map(|(index, _)| (index + 1).to_string())
        .collect();
    match failed.as_slice() {
        [] => None,
        [single] => Some(format!("Page {single} stayed an image (the AI could not read it in time or at all)")),
        many => Some(format!("Pages {} stayed images (the AI could not read it in time or at all)", many.join(", "))),
    }
}

fn blocker_text(blocker: RebuildBlocker, name: &str) -> String {
    match blocker {
        RebuildBlocker::NotPdf => "Only PDF files can be converted".to_owned(),
        RebuildBlocker::HasTextLayer => format!("{name} has real text already"),
        RebuildBlocker::AiNotReachable => "AI is not configured or cannot be reached: check Settings › AI".to_owned(),
        RebuildBlocker::ModelCannotReadImages => {
            "Your AI model cannot read images: choose a vision model in Settings › AI".to_owned()
        }
        RebuildBlocker::SourceChanged => format!("{name} changed while it was converted: convert it again"),
    }
}

#[cfg(test)]
mod tests;
