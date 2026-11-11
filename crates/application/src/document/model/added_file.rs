use dms_domain::document::Document;
use dms_domain::library::RelativePath;

use crate::common::OperationOutcome;
use crate::port::storage::ExternalFile;

/// What happened to one file the user added. `path` is where it was placed (with a suffix if the name was taken);
/// `duplicates` are documents that already had the same content.
#[derive(Debug, Clone, PartialEq)]
pub struct AddedFile {
    pub source: ExternalFile,
    pub outcome: OperationOutcome,
    pub path: Option<RelativePath>,
    pub duplicates: Vec<Document>,
}

impl AddedFile {
    #[must_use]
    pub fn failed(source: ExternalFile, outcome: OperationOutcome) -> Self {
        Self { source, outcome, path: None, duplicates: Vec::new() }
    }
}
