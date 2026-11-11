use crate::yaml::{Fields, MapWriter, Node, YamlError};

use super::owned;

/// Who confirmed a document's categorization last. `NON_EMPTY`; fields of a newer version are not kept here.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassificationDto {
    pub source: String,
    pub model: Option<String>,
    pub confirmed_at: String,
}

impl ClassificationDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            source: fields.text("source")?.to_owned(),
            model: owned(fields.opt_text("model")?),
            confirmed_at: fields.text("confirmedAt")?.to_owned(),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_text("source", Some(&self.source))
            .non_empty_text("model", self.model.as_deref())
            .non_empty_text("confirmedAt", Some(&self.confirmed_at))
            .build()
    }
}
