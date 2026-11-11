/// How an export is laid out: `flatten` puts every file in one folder (clashing names get a suffix),
/// `rename_to_title` names files after their title, `index` adds `index.csv` and `index.html`, `zip` writes one
/// ZIP file instead of a folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::struct_excessive_bools)] // four independent switches, as in Kotlin
pub struct ExportOptions {
    pub flatten: bool,
    pub rename_to_title: bool,
    pub index: bool,
    pub zip: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self { flatten: false, rename_to_title: false, index: true, zip: false }
    }
}
