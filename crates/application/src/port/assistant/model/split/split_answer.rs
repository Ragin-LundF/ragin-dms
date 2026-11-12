use dms_domain::split::SplitPart;

use crate::port::ai::ClassificationFailure;

/// The parts the AI proposes, from `model`, not yet checked against the page count; or why there are none.
#[derive(Debug, Clone, PartialEq)]
pub enum SplitAnswer {
    Proposed { parts: Vec<SplitPart>, model: String },
    Failed { reason: ClassificationFailure, detail: Option<String> },
}
