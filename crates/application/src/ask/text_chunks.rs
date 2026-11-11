use dms_domain::catalog::Catalog;
use dms_domain::document::Document;
use dms_domain::{iso_date, text};

use crate::ask::TextChunk;

/// Cuts a document's text into passages of about 800 tokens that overlap, at blanks where possible, and names the
/// document in a short header that goes in front of every passage, so a question about a title, a sender or a date
/// finds it too. Offsets and sizes count UTF-16 units like Kotlin's string indices.
pub struct TextChunks;

impl TextChunks {
    pub const SIZE: usize = 3_200;
    pub const OVERLAP: usize = 400;
    pub const MAX_CHUNKS: usize = 60;

    pub fn of(text: &str) -> Vec<TextChunk> {
        let units: Vec<u16> = text::trim(text).encode_utf16().collect();
        if units.is_empty() {
            return vec![TextChunk { start: 0, end: 0, text: String::new() }];
        }
        let length = units.len();
        let mut chunks = Vec::new();
        let mut start = 0;
        while start < length && chunks.len() < Self::MAX_CHUNKS {
            let hard_end = (start + Self::SIZE).min(length);
            let blank = last_blank_at_or_before(&units, hard_end).filter(|blank| *blank > start + Self::SIZE / 2);
            let end = if hard_end == length { hard_end } else { blank.unwrap_or(hard_end) };
            chunks.push(TextChunk { start, end, text: substring(&units, start, end) });
            if end == length {
                break;
            }
            start = end.saturating_sub(Self::OVERLAP).max(start + 1);
        }
        chunks
    }

    /// Title, correspondent, category, date, tags and note of `document`, one per line.
    pub fn header(document: &Document, catalog: &Catalog) -> String {
        let categories = catalog.categories();
        let mut path = Vec::new();
        let mut current = document.category_id.as_ref().and_then(|id| categories.get(id));
        while let Some(category) = current {
            if path.len() == categories.len() {
                break;
            }
            path.push(category.name.as_str());
            current = category.parent_id.as_ref().and_then(|id| categories.get(id));
        }
        path.reverse();
        let category = path.join(" / ");
        let mut tags: Vec<_> = document.tags.iter().collect();
        tags.sort();
        let lines = [
            Some(format!("Document: {}", document.metadata.title.as_deref().unwrap_or(document.path.file_name()))),
            document.metadata.correspondent.as_ref().map(|from| format!("From: {from}")),
            (!category.is_empty()).then(|| format!("Category: {category}")),
            document.metadata.document_date.map(|date| format!("Date: {}", iso_date(date))),
            (!tags.is_empty())
                .then(|| format!("Tags: {}", tags.iter().map(|tag| tag.value()).collect::<Vec<_>>().join(", "))),
            document.note.as_ref().map(|note| format!("Note: {note}")),
        ];
        lines.into_iter().flatten().collect::<Vec<_>>().join("\n")
    }
}

/// Kotlin's `lastIndexOf(' ', startIndex)`: the last blank at or before `from` (clamped to the text).
fn last_blank_at_or_before(units: &[u16], from: usize) -> Option<usize> {
    let last = from.min(units.len() - 1);
    units[..=last].iter().rposition(|unit| *unit == u16::from(b' '))
}

/// Kotlin's `substring` on UTF-16 indices; a cut through a surrogate pair becomes U+FFFD, where Kotlin kept half a
/// character.
pub(crate) fn substring(units: &[u16], start: usize, end: usize) -> String {
    String::from_utf16_lossy(&units[start..end])
}
