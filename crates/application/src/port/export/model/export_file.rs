/// A file an export writes from memory, such as the missing-items page of a tax export, at [`Self::target`] in the
/// bundle.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExportFile {
    pub target: String,
    pub content: Vec<u8>,
}
