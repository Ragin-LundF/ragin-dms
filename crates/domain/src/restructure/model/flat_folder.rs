use crate::library::RelativePath;

/// A folder with many documents and no subfolders, which the restructure assistant could sort.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlatFolder {
    pub path: RelativePath,
    pub file_count: usize,
}
