//! The prompts of the document assistant.

use dms_application::port::ai::DocumentContent;
use dms_application::port::assistant::AssistantRequest;
use dms_domain::taxonomy::TagName;
use dms_domain::text;
use indexmap::IndexSet;

use crate::prompt::{self, tax_tag_of};

const MAX_LISTED: usize = 300;

pub(crate) fn summary() -> String {
    prompt::SUMMARY.text()
}

pub(crate) fn summary_request(request: &AssistantRequest) -> String {
    let document = document_of(request);
    prompt::SUMMARY_REQUEST.fill(&[("fileName", &request.file_name), ("document", &document)])
}

pub(crate) fn tags(existing_tags: &IndexSet<TagName>) -> String {
    let mut names: Vec<&str> = existing_tags.iter().map(TagName::value).collect();
    names.sort_by(|first, second| text::compare(first, second));
    let listed = if names.is_empty() {
        "none yet".to_owned()
    } else {
        names.iter().take(MAX_LISTED).copied().collect::<Vec<_>>().join(", ")
    };
    prompt::TAGS.fill(&[("taxTag", &tax_tag_of(names.iter().copied())), ("tags", &listed)])
}

pub(crate) fn tags_request(request: &AssistantRequest) -> String {
    let document = document_of(request);
    prompt::TAGS_REQUEST.fill(&[("fileName", &request.file_name), ("document", &document)])
}

/// The system message of a conversation. It holds the document's text; page images go with the first question.
pub(crate) fn chat(request: &AssistantRequest) -> String {
    let note = if matches!(request.content, DocumentContent::PageImages(_)) {
        "The document is attached as page images to the first question."
    } else {
        "The text of the document follows."
    };
    let document = document_of(request);
    prompt::CHAT.fill(&[("documentNote", note), ("fileName", &request.file_name), ("document", &document)])
}

fn document_of(request: &AssistantRequest) -> String {
    match &request.content {
        DocumentContent::Text(text) => prompt::document_section(Some(text)),
        _ => prompt::document_section(None),
    }
}
