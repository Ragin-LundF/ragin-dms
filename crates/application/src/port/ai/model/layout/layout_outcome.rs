use dms_domain::rebuild::PageLayout;

use crate::port::ai::ClassificationFailure;

/// The layout of one page, or why the AI could not read it.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutOutcome {
    Read(PageLayout),
    Failure { reason: ClassificationFailure, detail: Option<String> },
}
