use indexmap::IndexMap;

use crate::catalog::dto::document_dto::{read_field_values, write_field_values};
use crate::catalog::dto::{FieldValueDto, extensible, owned, owned_texts, texts_node};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// `NON_EMPTY`
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RuleActionsDto {
    pub category_id: Option<String>,
    pub tags: Vec<String>,
    pub fields: IndexMap<String, FieldValueDto>,
    pub target_folder: Option<String>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(RuleActionsDto);

const KNOWN: [&str; 4] = ["categoryId", "tags", "fields", "targetFolder"];

impl RuleActionsDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            category_id: owned(fields.opt_text("categoryId")?),
            tags: owned_texts(fields, "tags")?,
            fields: read_field_values(fields, "fields")?,
            target_folder: owned(fields.opt_text("targetFolder")?),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_text("categoryId", self.category_id.as_deref())
            .non_empty_list("tags", texts_node(&self.tags))
            .non_empty_map("fields", write_field_values(&self.fields))
            .non_empty_text("targetFolder", self.target_folder.as_deref())
            .unknown(&self.unknown)
            .build()
    }
}
