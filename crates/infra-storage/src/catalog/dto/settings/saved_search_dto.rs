use indexmap::IndexMap;

use crate::catalog::dto::extensible;
use crate::catalog::dto::settings::QueryDto;
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// Every property is written, the query also when it is empty (`{}`).
#[derive(Debug, Clone, PartialEq)]
pub struct SavedSearchDto {
    pub id: String,
    pub name: String,
    pub query: QueryDto,
    pub unknown: IndexMap<String, Node>,
}

extensible!(SavedSearchDto);

const KNOWN: [&str; 3] = ["id", "name", "query"];

impl SavedSearchDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            id: fields.text("id")?.to_owned(),
            name: fields.text("name")?.to_owned(),
            query: fields.get("query").map(QueryDto::read).transpose()?.unwrap_or_default(),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .text("id", &self.id)
            .text("name", &self.name)
            .node("query", self.query.write())
            .unknown(&self.unknown)
            .build()
    }
}
