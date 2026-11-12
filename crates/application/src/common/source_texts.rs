use dms_domain::document::Document;
use dms_domain::library::LibraryFolder;
use dms_domain::text;

use crate::port::ai::DocumentContentReader;
use crate::port::storage::TextStore;

/// The texts of `sources` for the AI, in their order, each headed by its name and date, together cut to `budget`
/// UTF-16 units: the AI's transcription and the text layer. Documents the AI never reads and documents without text
/// are left out, so each text comes with its document.
pub fn source_texts<'d>(
    texts: &dyn TextStore,
    reader: &dyn DocumentContentReader,
    folder: &LibraryFolder,
    sources: &[&'d Document],
    budget: usize,
) -> Vec<(&'d Document, String)> {
    let mut budget = budget;
    let mut found = Vec::new();
    for document in sources.iter().copied().filter(|document| document.kind().readable_by_ai()) {
        let stored = texts.read(folder, &document.id);
        let extracted = reader.text(folder, document);
        let joined = [stored, extracted].into_iter().flatten().collect::<Vec<_>>().join("\n");
        if text::is_blank(&joined) || budget == 0 {
            continue;
        }
        let taken = text::take_utf16(&joined, budget);
        budget -= text::utf16_len(taken);
        found.push((document, format!("{}:\n{taken}", heading_of(document))));
    }
    found
}

fn heading_of(document: &Document) -> String {
    let name = document.metadata.title.as_deref().unwrap_or_else(|| document.path.file_name());
    match document.metadata.document_date {
        Some(date) => format!("{name} ({date})"),
        None => name.to_owned(),
    }
}
