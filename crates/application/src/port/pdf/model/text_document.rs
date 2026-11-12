use crate::port::pdf::TextBlock;

/// A plain text document on A4 pages, such as a letter or an overview: its blocks run down the pages, which break
/// on their own. A document without blocks is one empty page.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct TextDocument {
    pub blocks: Vec<TextBlock>,
}
