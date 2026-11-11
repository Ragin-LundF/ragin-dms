/// What moving a flat file into its subfolder did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidecarMove {
    Moved,
    /// The subfolder already holds a file for the document; both were left as they are.
    KeptExisting {
        /// Whether both files have the same content.
        identical: bool,
    },
}
