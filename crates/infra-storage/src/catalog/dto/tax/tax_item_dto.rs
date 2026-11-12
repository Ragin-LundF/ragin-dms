use indexmap::IndexMap;

use crate::catalog::dto::{extensible, only_true, owned};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// One item of a tax year's checklist; `addedByUser` only when true. `NON_EMPTY`.
#[derive(Debug, Clone, PartialEq)]
pub struct TaxItemDto {
    pub id: String,
    pub area: String,
    pub label: String,
    pub correspondent: Option<String>,
    pub document_id: Option<String>,
    pub state: String,
    pub note: Option<String>,
    pub added_by_user: bool,
    pub unknown: IndexMap<String, Node>,
}

extensible!(TaxItemDto);

const KNOWN: [&str; 8] = ["id", "area", "label", "correspondent", "documentId", "state", "note", "addedByUser"];

impl TaxItemDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            id: fields.text("id")?.to_owned(),
            area: fields.text("area")?.to_owned(),
            label: fields.text("label")?.to_owned(),
            correspondent: owned(fields.opt_text("correspondent")?),
            document_id: owned(fields.opt_text("documentId")?),
            state: fields.opt_text("state")?.unwrap_or("missing").to_owned(),
            note: owned(fields.opt_text("note")?),
            added_by_user: fields.bool_or("addedByUser", false)?,
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_text("id", Some(&self.id))
            .non_empty_text("area", Some(&self.area))
            .non_empty_text("label", Some(&self.label))
            .non_empty_text("correspondent", self.correspondent.as_deref())
            .non_empty_text("documentId", self.document_id.as_deref())
            .non_empty_text("state", Some(&self.state))
            .non_empty_text("note", self.note.as_deref())
            .opt_node("addedByUser", only_true(self.added_by_user))
            .unknown(&self.unknown)
            .build()
    }
}
