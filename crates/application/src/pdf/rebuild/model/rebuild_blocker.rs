/// Why a document cannot be rebuilt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RebuildBlocker {
    NotPdf,
    HasTextLayer,
    /// The AI server cannot be reached or is not set up.
    AiNotReachable,
    ModelCannotReadImages,
    /// The PDF changed while it was converted, so the pages would come from two versions.
    SourceChanged,
}
