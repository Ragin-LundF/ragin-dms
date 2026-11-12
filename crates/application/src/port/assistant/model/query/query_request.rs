use chrono::NaiveDate;
use dms_domain::field::FieldDefinition;
use dms_domain::tax::TaxArea;
use dms_domain::taxonomy::{Category, TagName};

/// A sentence to turn into filters, with today's date for words like "last year" and the names the library knows,
/// so the AI uses them instead of inventing new ones.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryRequest {
    pub sentence: String,
    pub today: NaiveDate,
    pub categories: Vec<Category>,
    pub tags: Vec<TagName>,
    pub correspondents: Vec<String>,
    pub fields: Vec<FieldDefinition>,
    pub tax_areas: Vec<TaxArea>,
}
