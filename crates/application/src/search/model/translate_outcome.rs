use crate::port::ai::ClassificationFailure;
use crate::search::QueryTranslation;

/// The filters a sentence stands for, or why there are none.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)] // returned once per sentence
pub enum TranslateOutcome {
    Translated(QueryTranslation),
    LibraryNotOpen,
    BlankSentence,
    Failed { reason: ClassificationFailure, detail: Option<String> },
}
