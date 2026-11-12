use indexmap::IndexMap;

use crate::catalog::dto::{extensible, owned};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// The user's decision about one sender's rhythm: `off` or `keep`. `NON_NULL`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpectationDto {
    pub correspondent: String,
    pub category_id: Option<String>,
    pub mode: String,
    pub unknown: IndexMap<String, Node>,
}

extensible!(ExpectationDto);

const KNOWN: [&str; 3] = ["correspondent", "categoryId", "mode"];

impl ExpectationDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            correspondent: fields.text("correspondent")?.to_owned(),
            category_id: owned(fields.opt_text("categoryId")?),
            mode: fields.text("mode")?.to_owned(),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .text("correspondent", &self.correspondent)
            .opt_text("categoryId", self.category_id.as_deref())
            .text("mode", &self.mode)
            .unknown(&self.unknown)
            .build()
    }
}
