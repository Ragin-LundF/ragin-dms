use crate::common::OperationOutcome;
use crate::pdf::rebuild::RebuildBlocker;
use crate::port::pdf::RebuiltFile;

/// What "Convert to real PDF" produced for one document.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // one per run
pub enum RebuildOutcome {
    /// The rebuilt PDF waits for the user's choice; the original is `original_size` bytes.
    Rebuilt { rebuilt: RebuiltFile, original_size: u64 },
    /// The document cannot be rebuilt, for this reason; nothing was written.
    NotAvailable(RebuildBlocker),
    /// The library is not open, the document is gone, or the PDF could not be read or written.
    Failed(OperationOutcome),
    /// The caller cancelled the run (Kotlin: coroutine cancellation); nothing was written.
    Cancelled,
}
