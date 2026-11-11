use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use dms_application::port::state::{ActivityEntry, ActivityLog};
use dms_application::port::storage::StorageResult;

use crate::catalog::dto::read_list;
use crate::enum_names::{self, ACTIVITY_KINDS};
use crate::instant;
use crate::state::dto::ActivityEntryDto;
use crate::yaml::{Fields, MapWriter, Node, YamlFile};

/// The last entries in `activity.yaml`, newest first. Entries of a kind this version does not know are kept in the
/// file and skipped when read.
pub struct FileActivityLog {
    file: YamlFile,
    capacity: usize,
    lock: Mutex<()>,
}

impl FileActivityLog {
    pub const DEFAULT_CAPACITY: usize = 200;

    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self::with_capacity(path, Self::DEFAULT_CAPACITY)
    }

    pub fn with_capacity(path: impl Into<PathBuf>, capacity: usize) -> Self {
        Self { file: YamlFile::new(path), capacity, lock: Mutex::new(()) }
    }

    fn entries(&self) -> StorageResult<Vec<ActivityEntryDto>> {
        let Some(node) = self.file.read()? else { return Ok(Vec::new()) };
        let fields = Fields::of(&node)?;
        Ok(read_list(fields, "entries", ActivityEntryDto::read)?)
    }
}

impl ActivityLog for FileActivityLog {
    fn record(&self, entry: &ActivityEntry) -> StorageResult<()> {
        let _guard = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
        let dto = ActivityEntryDto {
            at: instant::format(entry.at),
            kind: enum_names::name_of(ACTIVITY_KINDS, entry.kind).to_owned(),
            subject: entry.subject.clone(),
        };
        let mut entries = vec![dto];
        entries.extend(self.entries()?);
        entries.truncate(self.capacity);
        let node =
            MapWriter::new().node("entries", Node::Seq(entries.iter().map(ActivityEntryDto::write).collect())).build();
        self.file.write(&node)
    }

    fn recent(&self, limit: usize) -> StorageResult<Vec<ActivityEntry>> {
        let _guard = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
        let mut recent = Vec::new();
        for dto in self.entries()? {
            let Some(kind) = enum_names::value_of(ACTIVITY_KINDS, &dto.kind) else { continue };
            recent.push(ActivityEntry { at: instant::parse(&dto.at)?, kind, subject: dto.subject });
        }
        recent.truncate(limit);
        Ok(recent)
    }
}
