use indexmap::IndexMap;

use crate::rebuild::{KeepReason, RebuildRegion};

/// The result for one page: [`Self::lines_as_text`] of [`Self::lines_total`] lines became real text, the page was
/// straightened by [`Self::angle`] degrees, and [`Self::kept_reasons`] counts why the other lines stayed images.
#[derive(Debug, Clone, PartialEq)]
pub struct RebuiltPage {
    pub lines_as_text: usize,
    pub lines_total: usize,
    pub angle: f64,
    pub kept_reasons: IndexMap<KeepReason, usize>,
    pub regions: Vec<RebuildRegion>,
}
