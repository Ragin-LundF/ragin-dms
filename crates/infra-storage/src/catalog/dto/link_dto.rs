use indexmap::IndexMap;

use crate::yaml::{Fields, MapWriter, Node, YamlError};

use super::{extensible, owned};

/// A link between two documents, by id; `first` is the smaller id. `NON_NULL`.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkDto {
    pub first: String,
    pub second: String,
    pub label: Option<String>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(LinkDto);

const KNOWN: [&str; 3] = ["first", "second", "label"];

impl LinkDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            first: fields.text("first")?.to_owned(),
            second: fields.text("second")?.to_owned(),
            label: owned(fields.opt_text("label")?),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .text("first", &self.first)
            .text("second", &self.second)
            .opt_text("label", self.label.as_deref())
            .unknown(&self.unknown)
            .build()
    }
}
