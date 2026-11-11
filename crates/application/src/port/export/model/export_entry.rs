use dms_domain::library::RelativePath;

/// One file of an export: the document at [`Self::source`] in the library, written to [`Self::target`] in the
/// bundle.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExportEntry {
    pub source: RelativePath,
    pub target: String,
}
