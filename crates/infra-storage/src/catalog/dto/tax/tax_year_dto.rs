use indexmap::IndexMap;

use crate::catalog::dto::tax::TaxItemDto;
use crate::catalog::dto::{extensible, owned, read_list};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// The checklist of one tax year. `NON_EMPTY`.
#[derive(Debug, Clone, PartialEq)]
pub struct TaxYearDto {
    pub year: i64,
    pub items: Vec<TaxItemDto>,
    pub sent_on: Option<String>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(TaxYearDto);

const KNOWN: [&str; 3] = ["year", "items", "sentOn"];

impl TaxYearDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            year: fields.i64("year")?,
            items: read_list(fields, "items", TaxItemDto::read)?,
            sent_on: owned(fields.opt_text("sentOn")?),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .int("year", self.year)
            .non_empty_list("items", self.items.iter().map(TaxItemDto::write).collect())
            .non_empty_text("sentOn", self.sent_on.as_deref())
            .unknown(&self.unknown)
            .build()
    }
}
