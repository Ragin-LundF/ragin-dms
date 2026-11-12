use dms_domain::document::DocumentId;

use crate::common::OperationOutcome;

/// What became of each mail, in the order given.
#[derive(Debug, Clone, PartialEq)]
pub enum ImportEmailsOutcome {
    LibraryNotOpen,
    Imported(Vec<ImportedEmail>),
}

/// One mail: `source` names it for messages; `mail` and `attachments` are the new documents when `outcome` is done.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportedEmail {
    pub source: String,
    pub outcome: OperationOutcome,
    pub mail: Option<DocumentId>,
    pub attachments: Vec<DocumentId>,
}

impl ImportedEmail {
    pub(crate) fn failed(source: String, outcome: OperationOutcome) -> Self {
        Self { source, outcome, mail: None, attachments: Vec::new() }
    }
}
