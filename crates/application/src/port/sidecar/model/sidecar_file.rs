use dms_domain::document::DocumentId;

use crate::port::sidecar::SidecarKind;

/// One per-document file of a library folder's hidden folders.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SidecarFile {
    pub kind: SidecarKind,
    pub document_id: DocumentId,
}
