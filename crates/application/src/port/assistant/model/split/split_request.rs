use dms_domain::taxonomy::Category;

use crate::port::assistant::SplitPage;

/// What the AI needs to split a stack: every page in order and the library's categories, so it prefers them.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitRequest {
    pub pages: Vec<SplitPage>,
    pub categories: Vec<Category>,
}
