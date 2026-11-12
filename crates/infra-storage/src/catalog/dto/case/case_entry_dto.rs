use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// A document of a case, received or sent on `date`.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseEntryDto {
    pub document_id: String,
    pub direction: String,
    pub date: String,
}

impl CaseEntryDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            document_id: fields.text("documentId")?.to_owned(),
            direction: fields.text("direction")?.to_owned(),
            date: fields.text("date")?.to_owned(),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .text("documentId", &self.document_id)
            .text("direction", &self.direction)
            .text("date", &self.date)
            .build()
    }
}
