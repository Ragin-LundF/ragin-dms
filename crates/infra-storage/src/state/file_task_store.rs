use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use chrono::NaiveDate;
use dms_application::port::state::TaskStore;
use dms_application::port::storage::StorageResult;
use dms_domain::catalog::CatalogMergeConflict;
use dms_domain::classification::ClassificationSuggestion;

use dms_domain::document::DocumentId;
use dms_domain::expectation::ExpectationSnoozes;
use dms_domain::task::TaskKey;
use dms_domain::text;
use indexmap::{IndexMap, IndexSet};

use crate::catalog::dto::{owned_texts, read_list};
use crate::state::dto::SuggestionDto;
use crate::state::suggestion_mapper;
use crate::yaml::{Fields, MapWriter, Node, YamlError, YamlFile};

/// Pending suggestions and dismissed tasks in `tasks.yaml`. The conflicts of the last catalog merge are kept in
/// memory only: every opening of the library computes them again.
pub struct FileTaskStore {
    file: YamlFile,
    lock: Mutex<()>,
    merge_conflicts: Mutex<Vec<CatalogMergeConflict>>,
}

/// `tasks.yaml`: pending suggestions and the keys of dismissed tasks, both always written, and the snoozed tasks
/// with their day (2.1, only written when there are any).
#[derive(Debug, Default)]
struct TasksDto {
    suggestions: Vec<SuggestionDto>,
    dismissed: Vec<String>,
    snoozed: IndexMap<String, String>,
}

impl TasksDto {
    fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            suggestions: read_list(fields, "suggestions", SuggestionDto::read)?,
            dismissed: owned_texts(fields, "dismissed")?,
            snoozed: fields
                .entries("snoozed")?
                .into_iter()
                .filter_map(|(key, day)| Some((key.to_owned(), day.scalar_text()?.to_owned())))
                .collect(),
        })
    }

    fn write(&self) -> Node {
        MapWriter::new()
            .node("suggestions", Node::Seq(self.suggestions.iter().map(SuggestionDto::write).collect()))
            .node("dismissed", Node::Seq(self.dismissed.iter().map(Node::text).collect()))
            .non_empty_map("snoozed", self.snoozed.iter().map(|(key, day)| (key.clone(), Node::text(day))).collect())
            .build()
    }
}

impl FileTaskStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { file: YamlFile::new(path), lock: Mutex::new(()), merge_conflicts: Mutex::new(Vec::new()) }
    }

    fn read(&self) -> StorageResult<TasksDto> {
        Ok(self.file.read()?.map(|node| TasksDto::read(&node)).transpose()?.unwrap_or_default())
    }

    fn update(&self, change: impl FnOnce(&mut TasksDto)) -> StorageResult<()> {
        let _guard = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
        let mut tasks = self.read()?;
        change(&mut tasks);
        self.file.write(&tasks.write())
    }

    fn locked_read(&self) -> StorageResult<TasksDto> {
        let _guard = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
        self.read()
    }
}

impl TaskStore for FileTaskStore {
    fn save_suggestion(&self, suggestion: &ClassificationSuggestion) -> StorageResult<()> {
        let dto = suggestion_mapper::to_dto(suggestion);
        self.update(|tasks| {
            tasks.suggestions.retain(|other| other.document_id != dto.document_id);
            tasks.suggestions.push(dto);
        })
    }

    fn suggestions(&self) -> StorageResult<Vec<ClassificationSuggestion>> {
        let tasks = self.locked_read()?;
        Ok(tasks.suggestions.iter().map(suggestion_mapper::to_domain).collect::<Result<_, _>>()?)
    }

    fn remove_suggestion(&self, document_id: &DocumentId) -> StorageResult<()> {
        self.update(|tasks| tasks.suggestions.retain(|other| other.document_id != document_id.value()))
    }

    fn replace_merge_conflicts(&self, conflicts: &[CatalogMergeConflict]) -> StorageResult<()> {
        *self.merge_conflicts.lock().unwrap_or_else(PoisonError::into_inner) = conflicts.to_vec();
        Ok(())
    }

    fn merge_conflicts(&self) -> StorageResult<Vec<CatalogMergeConflict>> {
        Ok(self.merge_conflicts.lock().unwrap_or_else(PoisonError::into_inner).clone())
    }

    fn dismiss(&self, key: &TaskKey) -> StorageResult<()> {
        self.update(|tasks| {
            tasks.dismissed.push(key.value().to_owned());
            let mut distinct: Vec<String> = tasks.dismissed.drain(..).collect::<IndexSet<_>>().into_iter().collect();
            distinct.sort_by(|first, second| text::compare(first, second));
            tasks.dismissed = distinct;
        })
    }

    fn dismissed_keys(&self) -> StorageResult<IndexSet<TaskKey>> {
        Ok(self.locked_read()?.dismissed.into_iter().map(TaskKey::new).collect())
    }

    fn snooze(&self, key: &TaskKey, until: NaiveDate) -> StorageResult<()> {
        self.update(|tasks| {
            tasks.snoozed.insert(key.value().to_owned(), dms_domain::iso_date(until));
        })
    }

    fn snoozes(&self) -> StorageResult<ExpectationSnoozes> {
        let snoozed = self.locked_read()?.snoozed;
        // A day that cannot be read hides nothing: the task simply shows again.
        let snoozed_until = snoozed
            .into_iter()
            .filter_map(|(key, day)| Some((TaskKey::new(key), dms_domain::parse_iso_date(&day)?)))
            .collect();
        Ok(ExpectationSnoozes { snoozed_until })
    }
}
