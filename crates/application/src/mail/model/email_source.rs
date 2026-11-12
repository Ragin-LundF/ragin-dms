use dms_domain::document::DocumentId;
use dms_domain::library::RelativePath;

use crate::port::storage::ExternalFile;

/// A mail to import: a catalogued `.eml` document (for example in the inbox), whose results go next to it, or a
/// file from outside the library, whose results go into `folder`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmailSource {
    Document(DocumentId),
    External { file: ExternalFile, folder: RelativePath },
}
