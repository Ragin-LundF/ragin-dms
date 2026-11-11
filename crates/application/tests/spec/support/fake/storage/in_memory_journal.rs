use dms_application::port::storage::{OperationJournal, PendingOperation, StorageResult};

use crate::support::Shared;

#[derive(Default)]
pub struct InMemoryJournal {
    pub operations: Shared<Vec<PendingOperation>>,
}

impl OperationJournal for InMemoryJournal {
    fn record(&self, operation: &PendingOperation) -> StorageResult<()> {
        self.operations.update(|operations| operations.push(operation.clone()));
        Ok(())
    }

    fn pending(&self) -> StorageResult<Vec<PendingOperation>> {
        Ok(self.operations.get())
    }

    fn remove(&self, id: &str) -> StorageResult<()> {
        self.operations.update(|operations| operations.retain(|operation| operation.id != id));
        Ok(())
    }
}
