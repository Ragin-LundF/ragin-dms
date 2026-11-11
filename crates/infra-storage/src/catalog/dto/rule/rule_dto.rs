use indexmap::IndexMap;

use crate::catalog::dto::rule::{RuleActionsDto, RuleConditionDto};
use crate::catalog::dto::{extensible, read_list};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// A matching rule; `origin` is `LEARNED` or `MANUAL`. Every property is written.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleDto {
    pub id: String,
    pub name: String,
    pub origin: String,
    pub enabled: bool,
    pub conditions: Vec<RuleConditionDto>,
    pub actions: RuleActionsDto,
    pub unknown: IndexMap<String, Node>,
}

extensible!(RuleDto);

const KNOWN: [&str; 6] = ["id", "name", "origin", "enabled", "conditions", "actions"];

impl RuleDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            id: fields.text("id")?.to_owned(),
            name: fields.text("name")?.to_owned(),
            origin: fields.text("origin")?.to_owned(),
            enabled: fields.bool_or("enabled", true)?,
            conditions: read_list(fields, "conditions", RuleConditionDto::read)?,
            actions: fields.get("actions").map(RuleActionsDto::read).transpose()?.unwrap_or_default(),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .text("id", &self.id)
            .text("name", &self.name)
            .text("origin", &self.origin)
            .bool("enabled", self.enabled)
            .node("conditions", Node::Seq(self.conditions.iter().map(RuleConditionDto::write).collect()))
            .node("actions", self.actions.write())
            .unknown(&self.unknown)
            .build()
    }
}
