use chrono::NaiveDate;

/// What a PDF carries about itself in its document information and XMP: title, author (the correspondent),
/// subject (the category), keywords (the tags) and the document date.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Vec<String>,
    pub date: Option<NaiveDate>,
}
