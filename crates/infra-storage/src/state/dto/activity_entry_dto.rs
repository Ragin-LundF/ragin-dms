use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// One entry of `activity.yaml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityEntryDto {
    pub at: String,
    pub kind: String,
    pub subject: String,
}

impl ActivityEntryDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            at: fields.text("at")?.to_owned(),
            kind: fields.text("kind")?.to_owned(),
            subject: fields.text("subject")?.to_owned(),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new().text("at", &self.at).text("kind", &self.kind).text("subject", &self.subject).build()
    }
}
