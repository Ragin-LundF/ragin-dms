use indexmap::IndexMap;

use crate::yaml::{Fields, MapWriter, Node, YamlError};

use super::{extensible, owned};

/// `NON_EMPTY`
#[derive(Debug, Clone, PartialEq)]
pub struct CategoryDto {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub retention_years: Option<i32>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(CategoryDto);

const KNOWN: [&str; 4] = ["id", "name", "parentId", "retentionYears"];

impl CategoryDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            id: fields.text("id")?.to_owned(),
            name: fields.text("name")?.to_owned(),
            parent_id: owned(fields.opt_text("parentId")?),
            retention_years: fields.opt_i32("retentionYears")?,
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_text("id", Some(&self.id))
            .non_empty_text("name", Some(&self.name))
            .non_empty_text("parentId", self.parent_id.as_deref())
            .opt_node("retentionYears", self.retention_years.map(Node::int))
            .unknown(&self.unknown)
            .build()
    }
}
