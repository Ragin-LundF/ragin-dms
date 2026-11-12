/// How an export is laid out: `flatten` puts every file in one folder (clashing names get a suffix),
/// `rename_to_title` names files after their title, `index` adds `index.csv` and `index.html`, `zip` writes one
/// ZIP file instead of a folder. `tax_year` makes it an export for the tax adviser: files grouped in one folder per
/// tax area, with a "Missing items" page and sheet of that year's checklist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::struct_excessive_bools)] // four independent switches, as in Kotlin
pub struct ExportOptions {
    pub flatten: bool,
    pub rename_to_title: bool,
    pub index: bool,
    pub zip: bool,
    pub tax_year: Option<i32>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self { flatten: false, rename_to_title: false, index: true, zip: false, tax_year: None }
    }
}
