use indexmap::IndexMap;

use crate::catalog::dto::organize::EmergencyItemDto;
use crate::catalog::dto::{extensible, owned, read_list};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// The emergency folder. `NON_EMPTY`.
#[derive(Debug, Clone, PartialEq)]
pub struct EmergencyFolderDto {
    pub items: Vec<EmergencyItemDto>,
    pub last_refreshed: Option<String>,
    pub refresh_months: i64,
    pub intro_text: Option<String>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(EmergencyFolderDto);

const KNOWN: [&str; 4] = ["items", "lastRefreshed", "refreshMonths", "introText"];

/// The refresh of a folder without `refreshMonths`.
const DEFAULT_REFRESH_MONTHS: i64 = 12;

impl EmergencyFolderDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            items: read_list(fields, "items", EmergencyItemDto::read)?,
            last_refreshed: owned(fields.opt_text("lastRefreshed")?),
            refresh_months: fields.opt_i64("refreshMonths")?.unwrap_or(DEFAULT_REFRESH_MONTHS),
            intro_text: owned(fields.opt_text("introText")?),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_list("items", self.items.iter().map(EmergencyItemDto::write).collect())
            .non_empty_text("lastRefreshed", self.last_refreshed.as_deref())
            .int("refreshMonths", self.refresh_months)
            .non_empty_text("introText", self.intro_text.as_deref())
            .unknown(&self.unknown)
            .build()
    }
}
