/// One block of a [`crate::port::pdf::TextDocument`], laid out from the top of the page down. Text wraps at the
/// page width; a line break (`\n`) in a text starts a new line.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextBlock {
    /// A bold title line.
    Heading(String),
    Paragraph(String),
    /// Label and value per row; labels in bold in their own column.
    KeyValueTable(Vec<(String, String)>),
    /// Columns of equal width; the header is bold and repeated on every page the table runs onto.
    Table {
        header: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    /// An empty line.
    Spacer,
}
