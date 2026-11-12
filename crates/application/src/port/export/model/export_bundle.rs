use crate::port::export::{ExportEntry, ExportFile, IndexRow, MissingItem};

/// What an export writes: the files and, if wanted, an index (`index.csv` and `index.html`) whose columns are the
/// fixed ones plus [`Self::field_labels`], in the order of [`IndexRow::fields`]. A tax export adds [`Self::files`]
/// written from memory (the missing-items page, first) and the [`Self::missing_items`] sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportBundle {
    pub entries: Vec<ExportEntry>,
    pub index: Option<Vec<IndexRow>>,
    pub field_labels: Vec<String>,
    pub files: Vec<ExportFile>,
    pub missing_items: Option<Vec<MissingItem>>,
}
