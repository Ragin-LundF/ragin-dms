use dms_domain::classification::ClassificationSuggestion;

use crate::port::ai::ClassificationFailure;

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // one outcome per suggested document
pub enum SuggestOutcome {
    /// The suggestion waits on the dashboard for the user's review.
    Suggested(ClassificationSuggestion),
    LibraryNotOpen,
    NotFound,
    /// The file type is never sent to the AI, or the file could not be read. `detail` is technical.
    NotReadable {
        detail: Option<String>,
    },
    Failed {
        reason: ClassificationFailure,
        detail: Option<String>,
    },
}
