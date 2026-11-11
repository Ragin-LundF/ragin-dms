//! Writing exports outside the library: a folder or a ZIP file, with an index as CSV and HTML.

mod file_export_writer;
pub mod index_files;

pub use file_export_writer::FileExportWriter;
