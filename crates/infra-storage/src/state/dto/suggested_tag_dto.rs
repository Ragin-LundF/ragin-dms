use crate::yaml::{Fields, MapWriter, Node, YamlError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestedTagDto {
    pub name: String,
    pub is_new: bool,
}

impl SuggestedTagDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        let is_new = fields.opt_bool("isNew")?.ok_or_else(|| YamlError("missing property 'isNew'".to_owned()))?;
        Ok(Self { name: fields.text("name")?.to_owned(), is_new })
    }

    pub fn write(&self) -> Node {
        MapWriter::new().text("name", &self.name).bool("isNew", self.is_new).build()
    }
}
