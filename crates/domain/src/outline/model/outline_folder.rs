use crate::library::RelativePath;
use crate::taxonomy::CategoryId;

/// One folder of a [`crate::outline::FolderOutline`]: how many documents it holds, directly or below it, its most
/// common category, a few sample titles, and how many subfolders were left out to keep the outline short.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutlineFolder {
    pub path: RelativePath,
    pub document_count: usize,
    pub top_category: Option<CategoryId>,
    pub sample_titles: Vec<String>,
    pub hidden_subfolders: usize,
}
