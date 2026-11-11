use std::sync::Arc;

use dms_application::port::pdf::OcrTool;
use dms_application::port::storage::{StorageError, StorageFailure, StorageResult};
use dms_domain::change::FileFacts;
use dms_domain::library::{LibraryFolder, RelativePath};

use crate::support::Shared;
use crate::support::fake::storage::FakeFolders;

/// An OCR program that is installed when `installed` says so, and marks every page as having text.
pub struct FakeOcr {
    folders: Arc<FakeFolders>,
    pub installed: Shared<bool>,
}

impl FakeOcr {
    pub fn new(folders: Arc<FakeFolders>) -> Self {
        Self { folders, installed: Shared::new(false) }
    }
}

impl OcrTool for FakeOcr {
    fn available(&self) -> bool {
        self.installed.get()
    }

    fn ocr(&self, folder: &LibraryFolder, source: &RelativePath, target: &RelativePath) -> StorageResult<FileFacts> {
        let content =
            self.folders.content(folder, source.value()).ok_or_else(|| StorageError::new(StorageFailure::NotFound))?;
        let recognized = content.split('|').map(|page| format!("{page}#ocr")).collect::<Vec<_>>().join("|");
        self.folders.put(folder, target.value(), &recognized);
        Ok(FakeFolders::facts_of(&recognized))
    }
}
