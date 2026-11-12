use std::path::PathBuf;

use dms_application::port::mail::{MailAttachment, MailConverter, MailMessage, MailSource};
use dms_application::port::storage::StorageResult;
use dms_domain::change::FileFacts;
use dms_domain::library::{LibraryFolder, RelativePath};

use crate::adapter::LibraryLocations;
use crate::adapter::convert::{file_facts, not_found, storage_error, unreachable};
use crate::mail::{self, MailFile};

/// Reads `.eml` files with `mail-parser` and writes their attachments as new files; the mail is never changed.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmlMailConverter {
    locations: LibraryLocations,
}

impl EmlMailConverter {
    fn file(self, folder: &LibraryFolder, source: &MailSource) -> StorageResult<PathBuf> {
        match source {
            MailSource::Library(path) => self.locations.existing(folder, path).ok_or_else(not_found),
            MailSource::External(file) => Ok(PathBuf::from(file.location())),
        }
    }

    fn mail(self, folder: &LibraryFolder, source: &MailSource) -> StorageResult<MailFile> {
        mail::read_mail(&self.file(folder, source)?).map_err(storage_error)
    }
}

impl MailConverter for EmlMailConverter {
    fn read(&self, folder: &LibraryFolder, source: &MailSource) -> StorageResult<MailMessage> {
        let mail = self.mail(folder, source)?;
        Ok(MailMessage {
            attachments: mail
                .attachments
                .iter()
                .map(|part| MailAttachment { name: part.name.clone(), size: part.content.len() as u64 })
                .collect(),
            from_name: mail.from_name,
            from_address: mail.from_address,
            to: mail.to,
            subject: mail.subject,
            sent_at: mail.sent_at,
            body: mail.body,
        })
    }

    fn write_attachment(
        &self,
        folder: &LibraryFolder,
        source: &MailSource,
        index: usize,
        target: &RelativePath,
    ) -> StorageResult<FileFacts> {
        let mail = self.mail(folder, source)?;
        let part = mail.attachments.get(index).ok_or_else(not_found)?;
        let output = self.locations.target(folder, target).ok_or_else(unreachable)?;
        mail::write_attachment(&part.content, &output).map(file_facts).map_err(storage_error)
    }
}
