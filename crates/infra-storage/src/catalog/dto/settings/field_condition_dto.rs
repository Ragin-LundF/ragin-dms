use crate::catalog::dto::FieldValueDto;
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// One field condition of a saved search, such as `amount GREATER_THAN 100`. `NON_NULL`; fields of a newer
/// version are not kept here.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldConditionDto {
    pub key: String,
    pub comparison: String,
    pub value: Option<FieldValueDto>,
}

impl FieldConditionDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            key: fields.text("key")?.to_owned(),
            comparison: fields.text("comparison")?.to_owned(),
            value: fields.get("value").map(FieldValueDto::read).transpose()?,
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .text("key", &self.key)
            .text("comparison", &self.comparison)
            .opt_node("value", self.value.as_ref().map(FieldValueDto::write))
            .build()
    }
}
