mod metadata;
mod pages;
mod pdf_date;
mod read;
mod text_layer;
mod text_writer;

pub use metadata::PdfMetadata;
pub use pages::PageChange;
pub(crate) use pages::{page_of, page_text, pages_in, render_rgb, save_new};
pub use pdf_date::PdfDate;
pub use read::DocumentContent;
pub use text_writer::TextBlock;

#[cfg(test)]
mod tests;
