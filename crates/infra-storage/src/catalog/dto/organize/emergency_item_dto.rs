use indexmap::IndexMap;

use crate::catalog::dto::organize::EmergencyContactDto;
use crate::catalog::dto::{extensible, owned};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// One item of the emergency folder. Exactly one of `documentId`, `contractId`, `contact` and `note` is set, which
/// also tells its kind. `NON_NULL`.
#[derive(Debug, Clone, PartialEq)]
pub struct EmergencyItemDto {
    pub id: String,
    pub group: String,
    pub document_id: Option<String>,
    pub contract_id: Option<String>,
    pub contact: Option<EmergencyContactDto>,
    pub note: Option<String>,
    pub included: bool,
    pub recorded_hash: Option<String>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(EmergencyItemDto);

const KNOWN: [&str; 8] = ["id", "group", "documentId", "contractId", "contact", "note", "included", "recordedHash"];

impl EmergencyItemDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            id: fields.text("id")?.to_owned(),
            group: fields.opt_text("group")?.unwrap_or("other").to_owned(),
            document_id: owned(fields.opt_text("documentId")?),
            contract_id: owned(fields.opt_text("contractId")?),
            contact: fields.get("contact").map(EmergencyContactDto::read).transpose()?,
            note: owned(fields.opt_text("note")?),
            included: fields.bool_or("included", true)?,
            recorded_hash: owned(fields.opt_text("recordedHash")?),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .text("id", &self.id)
            .text("group", &self.group)
            .opt_text("documentId", self.document_id.as_deref())
            .opt_text("contractId", self.contract_id.as_deref())
            .opt_node("contact", self.contact.as_ref().map(EmergencyContactDto::write))
            .opt_text("note", self.note.as_deref())
            .bool("included", self.included)
            .opt_text("recordedHash", self.recorded_hash.as_deref())
            .unknown(&self.unknown)
            .build()
    }
}
