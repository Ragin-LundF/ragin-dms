/// One line of the bundle's index: where the file is in the bundle and what the catalog knows of it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexRow {
    pub path: String,
    pub title: String,
    pub date: String,
    pub correspondent: String,
    pub category: String,
    pub tags: String,
    pub fields: Vec<String>,
    pub note: String,
}
