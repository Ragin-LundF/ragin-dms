use indexmap::IndexMap;

use crate::catalog::dto::extensible;
use crate::catalog::dto::tax::MoneyDto;
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// A document's tax year, area and deductible part. `NON_NULL`.
#[derive(Debug, Clone, PartialEq)]
pub struct TaxAssignmentDto {
    pub year: i64,
    pub area: String,
    pub deductible: Option<MoneyDto>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(TaxAssignmentDto);

const KNOWN: [&str; 3] = ["year", "area", "deductible"];

impl TaxAssignmentDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            year: fields.i64("year")?,
            area: fields.text("area")?.to_owned(),
            deductible: fields.get("deductible").map(MoneyDto::read).transpose()?,
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .int("year", self.year)
            .text("area", &self.area)
            .opt_node("deductible", self.deductible.as_ref().map(MoneyDto::write))
            .unknown(&self.unknown)
            .build()
    }
}
