use crate::port::sidecar::SidecarFile;

/// A flat file left in place because its subfolder already holds a file for the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeptSidecarFile {
    pub file: SidecarFile,
    /// Whether both files have the same content, so the flat one is a redundant copy.
    pub identical: bool,
}
