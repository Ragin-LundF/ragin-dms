use crate::port::ai::{LayoutOutcome, LineOutcome, PageImage};

/// Reads the layout of a scanned page with the vision model the user configured: every printed text line with its
/// box, and the areas that are no text. In Kotlin, cancelling the caller cancelled the request; a blocking adapter
/// should keep its request timeout short enough for that.
pub trait PageLayoutReader: Send + Sync {
    fn read_layout(&self, page: &PageImage) -> LayoutOutcome;

    /// Reads one line again from `line`, a crop of the page, as plain text.
    fn read_line(&self, line: &PageImage) -> LineOutcome;
}
