use indexmap::IndexMap;

use crate::catalog::dto::{extensible, only_true, owned};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// A follow-up of a case; `source` is `manual` or `ai` with the document it came from, `done` only when true.
/// `NON_NULL`.
#[derive(Debug, Clone, PartialEq)]
pub struct FollowUpDto {
    pub id: String,
    pub date: String,
    pub text: String,
    pub done: bool,
    pub source: String,
    pub source_document_id: Option<String>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(FollowUpDto);

const KNOWN: [&str; 6] = ["id", "date", "text", "done", "source", "sourceDocumentId"];

impl FollowUpDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            id: fields.text("id")?.to_owned(),
            date: fields.text("date")?.to_owned(),
            text: fields.text("text")?.to_owned(),
            done: fields.bool_or("done", false)?,
            source: fields.opt_text("source")?.unwrap_or("manual").to_owned(),
            source_document_id: owned(fields.opt_text("sourceDocumentId")?),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .text("id", &self.id)
            .text("date", &self.date)
            .text("text", &self.text)
            .opt_node("done", only_true(self.done))
            .text("source", &self.source)
            .opt_text("sourceDocumentId", self.source_document_id.as_deref())
            .unknown(&self.unknown)
            .build()
    }
}
