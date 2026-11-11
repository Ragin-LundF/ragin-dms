//! The prompt templates in `prompts/`, compiled in. A template names its values as `{{name}}`; values are filled in
//! the given order, so put the document text last and text that contains `{{x}}` is never filled again.

mod tax_tag;

pub(crate) use tax_tag::tax_tag_of;

/// One prompt template, trimmed when it is filled, as Kotlin trimmed it on loading.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Template(&'static str);

impl Template {
    pub(crate) fn fill(self, values: &[(&str, &str)]) -> String {
        values
            .iter()
            .fold(self.0.trim().to_owned(), |text, (key, value)| text.replace(&format!("{{{{{key}}}}}"), value))
    }

    pub(crate) fn text(self) -> String {
        self.fill(&[])
    }
}

pub(crate) const ASK_LIBRARY: Template = Template(include_str!("../../prompts/ask-library.md"));
pub(crate) const CHAT: Template = Template(include_str!("../../prompts/chat.md"));
pub(crate) const CLASSIFICATION: Template = Template(include_str!("../../prompts/classification.md"));
pub(crate) const CLASSIFICATION_OUTLINE: Template = Template(include_str!("../../prompts/classification-outline.md"));
pub(crate) const CLASSIFICATION_OUTLINE_EMPTY: Template =
    Template(include_str!("../../prompts/classification-outline-empty.md"));
pub(crate) const CLASSIFICATION_OUTLINE_NONE: Template =
    Template(include_str!("../../prompts/classification-outline-none.md"));
pub(crate) const CLASSIFICATION_REQUEST: Template = Template(include_str!("../../prompts/classification-request.md"));
pub(crate) const CLASSIFICATION_RULES: Template = Template(include_str!("../../prompts/classification-rules.md"));
pub(crate) const DOCUMENT_IMAGES: Template = Template(include_str!("../../prompts/document-images.md"));
pub(crate) const DOCUMENT_TEXT: Template = Template(include_str!("../../prompts/document-text.md"));
pub(crate) const REBUILD_LAYOUT: Template = Template(include_str!("../../prompts/rebuild-layout.md"));
pub(crate) const REBUILD_LAYOUT_REQUEST: Template = Template(include_str!("../../prompts/rebuild-layout-request.md"));
pub(crate) const REBUILD_LINE: Template = Template(include_str!("../../prompts/rebuild-line.md"));
pub(crate) const SUMMARY: Template = Template(include_str!("../../prompts/summary.md"));
pub(crate) const SUMMARY_REQUEST: Template = Template(include_str!("../../prompts/summary-request.md"));
pub(crate) const TAGS: Template = Template(include_str!("../../prompts/tags.md"));
pub(crate) const TAGS_REQUEST: Template = Template(include_str!("../../prompts/tags-request.md"));

/// The document section of a request: the text as the reader cut it, or a note that `None` means page images.
pub(crate) fn document_section(text: Option<&str>) -> String {
    match text {
        Some(text) => DOCUMENT_TEXT.fill(&[("text", text)]),
        None => DOCUMENT_IMAGES.text(),
    }
}
