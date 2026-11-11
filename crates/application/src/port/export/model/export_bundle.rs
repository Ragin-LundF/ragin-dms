use crate::port::export::{ExportEntry, IndexRow};

/// What an export writes: the files and, if wanted, an index (`index.csv` and `index.html`) whose columns are the
/// fixed ones plus [`Self::field_labels`], in the order of [`IndexRow::fields`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportBundle {
    pub entries: Vec<ExportEntry>,
    pub index: Option<Vec<IndexRow>>,
    pub field_labels: Vec<String>,
}
