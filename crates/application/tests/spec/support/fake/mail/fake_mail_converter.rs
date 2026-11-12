use std::collections::HashMap;
use std::sync::Arc;

use dms_application::port::mail::{MailConverter, MailMessage, MailSource};
use dms_application::port::storage::{StorageError, StorageFailure, StorageResult};
use dms_domain::change::FileFacts;
use dms_domain::library::{LibraryFolder, RelativePath};

use crate::support::Shared;
use crate::support::fake::storage::FakeFolders;

/// Mails scripted by file name; an attachment is written as the text `attachment <name>` into `FakeFolders`.
pub struct FakeMailConverter {
    folders: Arc<FakeFolders>,
    pub mails: Shared<HashMap<String, MailMessage>>,
}

impl FakeMailConverter {
    pub fn new(folders: Arc<FakeFolders>) -> Self {
        Self { folders, mails: Shared::default() }
    }

    pub fn set(&self, file_name: &str, mail: MailMessage) {
        self.mails.update(|mails| mails.insert(file_name.to_owned(), mail));
    }
}

fn file_name(source: &MailSource) -> &str {
    match source {
        MailSource::Library(path) => path.file_name(),
        MailSource::External(file) => file.file_name(),
    }
}

impl MailConverter for FakeMailConverter {
    fn read(&self, _folder: &LibraryFolder, source: &MailSource) -> StorageResult<MailMessage> {
        self.mails.lock().get(file_name(source)).cloned().ok_or_else(|| StorageError::new(StorageFailure::NotFound))
    }

    fn write_attachment(
        &self,
        folder: &LibraryFolder,
        source: &MailSource,
        index: usize,
        target: &RelativePath,
    ) -> StorageResult<FileFacts> {
        let mail = self.read(folder, source)?;
        let attachment = mail.attachments.get(index).ok_or_else(|| StorageError::new(StorageFailure::NotFound))?;
        if self.folders.content(folder, target.value()).is_some() {
            return Err(StorageError::new(StorageFailure::AlreadyExists));
        }
        let content = format!("attachment {}", attachment.name);
        self.folders.put(folder, target.value(), &content);
        Ok(FakeFolders::facts_of(&content))
    }
}
