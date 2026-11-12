use dms_domain::change::FileFacts;
use dms_domain::library::{LibraryFolder, RelativePath};

use crate::port::mail::{MailMessage, MailSource};
use crate::port::storage::StorageResult;

/// Reads email files (RFC 5322, `.eml`) and writes their attachments as files of their own. The mail itself becomes
/// a PDF through [`crate::port::pdf::PdfEditor::write_text_document`]. Sources are only read; every attachment is a
/// new file at a target that must not exist yet.
///
/// Every `# Errors` case is a [`crate::port::storage::StorageError`]: the file is missing or no readable mail, the
/// target exists, or the file system failed.
pub trait MailConverter: Send + Sync {
    /// The mail at `source`; a library source is read from `folder`.
    ///
    /// # Errors
    /// See the trait documentation.
    fn read(&self, folder: &LibraryFolder, source: &MailSource) -> StorageResult<MailMessage>;

    /// Writes attachment number `index` of [`MailMessage::attachments`] to `target` in `folder`.
    ///
    /// # Errors
    /// See the trait documentation.
    fn write_attachment(
        &self,
        folder: &LibraryFolder,
        source: &MailSource,
        index: usize,
        target: &RelativePath,
    ) -> StorageResult<FileFacts>;
}
