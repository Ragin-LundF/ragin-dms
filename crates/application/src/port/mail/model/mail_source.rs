use dms_domain::library::RelativePath;

use crate::port::storage::ExternalFile;

/// Where a mail file is: in the library (for example in the inbox) or outside it (dropped from the file manager).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MailSource {
    Library(RelativePath),
    External(ExternalFile),
}
