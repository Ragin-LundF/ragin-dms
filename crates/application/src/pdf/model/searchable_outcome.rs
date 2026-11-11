use crate::common::OperationOutcome;

/// What "Make searchable" did with one document.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // one per document
pub enum SearchableOutcome {
    /// The new version with a text layer replaced the file; the outcome says how the write went.
    Made(OperationOutcome),
    /// The PDF has a text layer already; nothing changed.
    AlreadySearchable,
    /// There is no transcription to add yet; let the AI read the scan first, or use precise OCR.
    NoTranscription,
    /// Precise OCR was asked for, but the OCR program is not installed.
    OcrUnavailable,
    NotPdf,
    /// The library is not open, the document is gone, or the PDF could not be read.
    Failed(OperationOutcome),
}
