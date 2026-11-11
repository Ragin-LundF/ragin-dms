use indexmap::IndexMap;

use crate::catalog::dto::{extensible, owned};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// A custom field: key, label, type (`TEXT`, `NUMBER`, `MONEY`, `DATE`, `BOOLEAN`) and deadline role. `NON_NULL`.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinitionDto {
    pub key: String,
    pub label: String,
    pub field_type: String,
    pub role: Option<String>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(FieldDefinitionDto);

const KNOWN: [&str; 4] = ["key", "label", "type", "role"];

impl FieldDefinitionDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            key: fields.text("key")?.to_owned(),
            label: fields.text("label")?.to_owned(),
            field_type: fields.text("type")?.to_owned(),
            role: owned(fields.opt_text("role")?),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .text("key", &self.key)
            .text("label", &self.label)
            .text("type", &self.field_type)
            .opt_text("role", self.role.as_deref())
            .unknown(&self.unknown)
            .build()
    }
}
