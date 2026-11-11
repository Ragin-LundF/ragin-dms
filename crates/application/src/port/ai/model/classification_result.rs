use dms_domain::classification::ClassificationSuggestion;

use crate::port::ai::ClassificationFailure;

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // one result per classified document
pub enum ClassificationResult {
    Suggested(ClassificationSuggestion),
    Failed { reason: ClassificationFailure, detail: Option<String> },
}
