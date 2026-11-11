use indexmap::IndexMap;

use crate::catalog::dto::{extensible, owned_texts, texts_node};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// `NON_EMPTY`
#[derive(Debug, Clone, PartialEq)]
pub struct CorrespondentDto {
    pub name: String,
    pub aliases: Vec<String>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(CorrespondentDto);

const KNOWN: [&str; 2] = ["name", "aliases"];

impl CorrespondentDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            name: fields.text("name")?.to_owned(),
            aliases: owned_texts(fields, "aliases")?,
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_text("name", Some(&self.name))
            .non_empty_list("aliases", texts_node(&self.aliases))
            .unknown(&self.unknown)
            .build()
    }
}
