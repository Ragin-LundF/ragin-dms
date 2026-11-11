use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use dms_application::port::storage::{OperationJournal, PendingOperation, StorageResult};

use crate::state::operation_mapper;
use crate::yaml::{Fields, MapWriter, Node, YamlError, YamlFile};

/// The failed-operation journal in `journal.yaml`, oldest first. Every change is written before the call returns.
/// Entries of an unknown type are skipped on read and so dropped on the next write, as in the Kotlin version.
pub struct FileOperationJournal {
    file: YamlFile,
    lock: Mutex<()>,
}

impl FileOperationJournal {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { file: YamlFile::new(path), lock: Mutex::new(()) }
    }

    fn read(&self) -> StorageResult<Vec<PendingOperation>> {
        let Some(node) = self.file.read()? else { return Ok(Vec::new()) };
        let operations = Fields::of(&node)?
            .list("operations")?
            .iter()
            .filter_map(|item| operation_mapper::to_domain(item).transpose())
            .collect::<Result<Vec<_>, YamlError>>()?;
        Ok(operations)
    }

    fn update(&self, change: impl FnOnce(&mut Vec<PendingOperation>)) -> StorageResult<()> {
        let _guard = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
        let mut operations = self.read()?;
        change(&mut operations);
        let node = MapWriter::new()
            .node("operations", Node::Seq(operations.iter().map(operation_mapper::to_node).collect()))
            .build();
        self.file.write(&node)
    }
}

impl OperationJournal for FileOperationJournal {
    fn record(&self, operation: &PendingOperation) -> StorageResult<()> {
        self.update(|operations| operations.push(operation.clone()))
    }

    fn pending(&self) -> StorageResult<Vec<PendingOperation>> {
        let _guard = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
        self.read()
    }

    fn remove(&self, id: &str) -> StorageResult<()> {
        self.update(|operations| operations.retain(|operation| operation.id != id))
    }
}
