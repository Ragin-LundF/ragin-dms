use crate::yaml::{Fields, MapWriter, Node, YamlError};

#[derive(Debug, Clone, PartialEq)]
pub struct CaseNoteDto {
    pub date: String,
    pub text: String,
}

impl CaseNoteDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self { date: fields.text("date")?.to_owned(), text: fields.text("text")?.to_owned() })
    }

    pub fn write(&self) -> Node {
        MapWriter::new().text("date", &self.date).text("text", &self.text).build()
    }
}
