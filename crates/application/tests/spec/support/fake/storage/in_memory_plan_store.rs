use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use dms_application::port::plan::{PlanStore, RestructureRecord};
use dms_application::port::storage::{StorageError, StorageFailure, StorageResult};
use dms_domain::library::{LibraryFolder, LibraryFolderId};
use dms_domain::text;
use indexmap::IndexMap;

use crate::support::Shared;

/// `.dms-plans/` of every library folder, in memory; folders in `unreachable` fail like in `FakeFolders`.
pub struct InMemoryPlanStore {
    unreachable: Arc<Shared<HashSet<LibraryFolderId>>>,
    pub records: Shared<HashMap<LibraryFolderId, IndexMap<String, RestructureRecord>>>,
}

impl InMemoryPlanStore {
    pub fn new(unreachable: Arc<Shared<HashSet<LibraryFolderId>>>) -> Self {
        Self { unreachable, records: Shared::default() }
    }

    pub fn records_in(&self, folder: &LibraryFolder) -> IndexMap<String, RestructureRecord> {
        self.records.lock().get(&folder.id).cloned().unwrap_or_default()
    }
}

impl PlanStore for InMemoryPlanStore {
    fn write(&self, folder: &LibraryFolder, record: &RestructureRecord) -> StorageResult<()> {
        if self.unreachable.lock().contains(&folder.id) {
            return Err(StorageError::new(StorageFailure::FolderUnreachable));
        }
        self.records.update(|records| {
            records.entry(folder.id.clone()).or_default().insert(record.name.clone(), record.clone())
        });
        Ok(())
    }

    fn read(&self, folder: &LibraryFolder, name: &str) -> Option<RestructureRecord> {
        if self.unreachable.lock().contains(&folder.id) {
            return None;
        }
        self.records.lock().get(&folder.id).and_then(|records| records.get(name)).cloned()
    }

    fn names(&self, folder: &LibraryFolder) -> Vec<String> {
        let mut names: Vec<String> =
            self.records.lock().get(&folder.id).map(|records| records.keys().cloned().collect()).unwrap_or_default();
        names.sort_by(|first, second| text::compare(second, first));
        names
    }
}
