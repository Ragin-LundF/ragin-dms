use indexmap::IndexMap;

use crate::yaml::{Fields, MapWriter, Node, YamlError};

use super::{extensible, owned};

/// The opt-in retention of one document: a keep-until day and when the user marked it done. `NON_NULL`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RetentionDto {
    pub keep_until: Option<String>,
    pub settled_at: Option<String>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(RetentionDto);

const KNOWN: [&str; 2] = ["keepUntil", "settledAt"];

impl RetentionDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            keep_until: owned(fields.opt_text("keepUntil")?),
            settled_at: owned(fields.opt_text("settledAt")?),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .opt_text("keepUntil", self.keep_until.as_deref())
            .opt_text("settledAt", self.settled_at.as_deref())
            .unknown(&self.unknown)
            .build()
    }
}
