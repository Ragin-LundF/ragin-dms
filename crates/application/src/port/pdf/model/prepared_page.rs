use crate::port::ai::PageImage;

/// Page [`Self::page_index`] of a scan, rendered at full resolution and straightened by [`Self::angle`] degrees,
/// as PNG.
#[derive(Debug, Clone)]
pub struct PreparedPage {
    pub page_index: usize,
    pub image: PageImage,
    pub angle: f64,
}
