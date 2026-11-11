//! Between the engine's own types and the domain and port types, which the engine does not depend on.

use chrono::{Datelike, NaiveDate};
use dms_application::port::ai::{self, PageImage};
use dms_application::port::pdf;
use dms_application::port::storage::{StorageError, StorageFailure};
use dms_domain::change::FileFacts;
use dms_domain::document::ContentHash;
use dms_domain::rebuild as domain;

use crate::{PdfDate, PdfError, rebuild as engine};

pub(super) fn storage_error(error: PdfError) -> StorageError {
    match error {
        PdfError::NotFound => StorageError::new(StorageFailure::NotFound),
        PdfError::AlreadyExists => StorageError::new(StorageFailure::AlreadyExists),
        PdfError::UnsupportedFormat(detail) => StorageError::with_detail(StorageFailure::UnsupportedFormat, detail),
        other => StorageError::with_detail(StorageFailure::IoError, other.to_string()),
    }
}

pub(super) fn not_found() -> StorageError {
    StorageError::new(StorageFailure::NotFound)
}

pub(super) fn unreachable() -> StorageError {
    StorageError::new(StorageFailure::FolderUnreachable)
}

pub(super) fn file_facts(facts: crate::FileFacts) -> FileFacts {
    FileFacts { size: facts.size, content_hash: ContentHash::new(facts.content_hash).ok() }
}

pub(super) fn page_image(image: crate::PageImage) -> PageImage {
    PageImage { bytes: image.bytes, mime_type: image.mime_type.to_owned() }
}

pub(super) fn content(content: crate::DocumentContent) -> ai::DocumentContent {
    match content {
        crate::DocumentContent::Text(text) => ai::DocumentContent::Text(text),
        crate::DocumentContent::PageImages(pages) => {
            ai::DocumentContent::PageImages(pages.into_iter().map(page_image).collect())
        }
        crate::DocumentContent::NotReadable(reason) => ai::DocumentContent::NotReadable(reason),
    }
}

pub(super) fn page_change(change: &pdf::PageChange) -> crate::PageChange {
    crate::PageChange { page: change.page, rotation: change.rotation }
}

pub(super) fn metadata_to_engine(metadata: &pdf::PdfMetadata) -> crate::PdfMetadata {
    crate::PdfMetadata {
        title: metadata.title.clone(),
        author: metadata.author.clone(),
        subject: metadata.subject.clone(),
        keywords: metadata.keywords.clone(),
        date: metadata.date.map(|date| PdfDate { year: date.year(), month: date.month(), day: date.day() }),
    }
}

pub(super) fn metadata_from_engine(metadata: crate::PdfMetadata) -> pdf::PdfMetadata {
    pdf::PdfMetadata {
        title: metadata.title,
        author: metadata.author,
        subject: metadata.subject,
        keywords: metadata.keywords,
        date: metadata.date.and_then(|date| NaiveDate::from_ymd_opt(date.year, date.month, date.day)),
    }
}

/// The engine's prepared page; the picture is copied, because the engine takes its own type.
pub(super) fn prepared_to_engine(prepared: &pdf::PreparedPage) -> engine::PreparedPage {
    engine::PreparedPage {
        page_index: prepared.page_index,
        image: crate::PageImage { bytes: prepared.image.bytes.clone(), mime_type: crate::images::PNG },
        angle: prepared.angle,
    }
}

pub(super) fn prepared_from_engine(prepared: engine::PreparedPage) -> pdf::PreparedPage {
    pdf::PreparedPage { page_index: prepared.page_index, image: page_image(prepared.image), angle: prepared.angle }
}

pub(super) fn page_input(input: &pdf::PageInput) -> engine::PageInput {
    match input {
        pdf::PageInput::Read { prepared, layout } => {
            engine::PageInput::Read { prepared: prepared_to_engine(prepared), layout: layout_to_engine(layout) }
        }
        pdf::PageInput::Failed { page_index } => engine::PageInput::Failed { page_index: *page_index },
    }
}

pub(super) fn layout_box(bounds: &domain::LayoutBox) -> engine::LayoutBox {
    engine::LayoutBox { left: bounds.left, top: bounds.top, right: bounds.right, bottom: bounds.bottom }
}

fn layout_to_engine(layout: &domain::PageLayout) -> engine::PageLayout {
    engine::PageLayout {
        lines: layout
            .lines
            .iter()
            .map(|line| engine::LayoutLine {
                bounds: layout_box(&line.bounds),
                text: line.text.clone(),
                bold: line.bold,
                family: match line.family {
                    domain::FontFamily::Sans => engine::FontFamily::Sans,
                    domain::FontFamily::Serif => engine::FontFamily::Serif,
                    domain::FontFamily::Mono => engine::FontFamily::Mono,
                },
                legible: line.legible,
                kept: line.kept.map(keep_reason_to_engine),
            })
            .collect(),
        graphics: layout
            .graphics
            .iter()
            .map(|graphic| engine::LayoutGraphic { kind: graphic.kind.clone(), bounds: layout_box(&graphic.bounds) })
            .collect(),
    }
}

fn keep_reason_to_engine(reason: domain::KeepReason) -> engine::KeepReason {
    match reason {
        domain::KeepReason::NotSure => engine::KeepReason::NotSure,
        domain::KeepReason::NoInk => engine::KeepReason::NoInk,
        domain::KeepReason::Covered => engine::KeepReason::Covered,
        domain::KeepReason::NotEncodable => engine::KeepReason::NotEncodable,
        domain::KeepReason::TextMismatch => engine::KeepReason::TextMismatch,
        domain::KeepReason::LostCharacters => engine::KeepReason::LostCharacters,
        domain::KeepReason::PageFailed => engine::KeepReason::PageFailed,
    }
}

fn keep_reason_from_engine(reason: engine::KeepReason) -> domain::KeepReason {
    match reason {
        engine::KeepReason::NotSure => domain::KeepReason::NotSure,
        engine::KeepReason::NoInk => domain::KeepReason::NoInk,
        engine::KeepReason::Covered => domain::KeepReason::Covered,
        engine::KeepReason::NotEncodable => domain::KeepReason::NotEncodable,
        engine::KeepReason::TextMismatch => domain::KeepReason::TextMismatch,
        engine::KeepReason::LostCharacters => domain::KeepReason::LostCharacters,
        engine::KeepReason::PageFailed => domain::KeepReason::PageFailed,
    }
}

pub(super) fn rebuilt_page(page: engine::RebuiltPage) -> domain::RebuiltPage {
    domain::RebuiltPage {
        lines_as_text: page.lines_as_text,
        lines_total: page.lines_total,
        angle: page.angle,
        kept_reasons: page
            .kept_reasons
            .into_iter()
            .map(|(reason, count)| (keep_reason_from_engine(reason), count))
            .collect(),
        regions: page
            .regions
            .into_iter()
            .map(|region| domain::RebuildRegion {
                bounds: domain::LayoutBox {
                    left: region.bounds.left,
                    top: region.bounds.top,
                    right: region.bounds.right,
                    bottom: region.bounds.bottom,
                },
                kind: match region.kind {
                    engine::RegionKind::Text => domain::RegionKind::Text,
                    engine::RegionKind::KeptLine => domain::RegionKind::KeptLine,
                    engine::RegionKind::Image => domain::RegionKind::Image,
                },
            })
            .collect(),
    }
}

/// The engine's handle of a rebuilt file: the rebuilder finds the file by its id alone.
pub(super) fn rebuilt_handle(rebuilt: &pdf::RebuiltFile) -> engine::RebuiltFile {
    engine::RebuiltFile {
        id: rebuilt.id.clone(),
        pages: Vec::new(),
        size: rebuilt.size,
        source_hash: rebuilt.source_hash.value().to_owned(),
    }
}
