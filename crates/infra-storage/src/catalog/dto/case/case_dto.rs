use indexmap::IndexMap;

use crate::catalog::dto::case::{CaseEntryDto, CaseNoteDto, FollowUpDto};
use crate::catalog::dto::{extensible, owned, read_list};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// One case. `NON_EMPTY`.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseDto {
    pub id: String,
    pub title: String,
    pub correspondent: Option<String>,
    pub status: String,
    pub opened_on: String,
    pub closed_on: Option<String>,
    pub waiting_since: Option<String>,
    pub remind_after_days: i64,
    pub entries: Vec<CaseEntryDto>,
    pub notes: Vec<CaseNoteDto>,
    pub follow_ups: Vec<FollowUpDto>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(CaseDto);

const KNOWN: [&str; 11] = [
    "id",
    "title",
    "correspondent",
    "status",
    "openedOn",
    "closedOn",
    "waitingSince",
    "remindAfterDays",
    "entries",
    "notes",
    "followUps",
];

/// The reminder of a case without `remindAfterDays`.
const DEFAULT_REMIND_AFTER_DAYS: i64 = 21;

impl CaseDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            id: fields.text("id")?.to_owned(),
            title: fields.text("title")?.to_owned(),
            correspondent: owned(fields.opt_text("correspondent")?),
            status: fields.opt_text("status")?.unwrap_or("open").to_owned(),
            opened_on: fields.text("openedOn")?.to_owned(),
            closed_on: owned(fields.opt_text("closedOn")?),
            waiting_since: owned(fields.opt_text("waitingSince")?),
            remind_after_days: fields.opt_i64("remindAfterDays")?.unwrap_or(DEFAULT_REMIND_AFTER_DAYS),
            entries: read_list(fields, "entries", CaseEntryDto::read)?,
            notes: read_list(fields, "notes", CaseNoteDto::read)?,
            follow_ups: read_list(fields, "followUps", FollowUpDto::read)?,
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_text("id", Some(&self.id))
            .non_empty_text("title", Some(&self.title))
            .non_empty_text("correspondent", self.correspondent.as_deref())
            .non_empty_text("status", Some(&self.status))
            .non_empty_text("openedOn", Some(&self.opened_on))
            .non_empty_text("closedOn", self.closed_on.as_deref())
            .non_empty_text("waitingSince", self.waiting_since.as_deref())
            .int("remindAfterDays", self.remind_after_days)
            .non_empty_list("entries", self.entries.iter().map(CaseEntryDto::write).collect())
            .non_empty_list("notes", self.notes.iter().map(CaseNoteDto::write).collect())
            .non_empty_list("followUps", self.follow_ups.iter().map(FollowUpDto::write).collect())
            .unknown(&self.unknown)
            .build()
    }
}
