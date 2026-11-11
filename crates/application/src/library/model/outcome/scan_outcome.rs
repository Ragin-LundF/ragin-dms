use crate::library::ScanResult;

#[derive(Debug, Clone, PartialEq)]
pub enum ScanOutcome {
    Scanned(ScanResult),
    /// No library is open, or it was closed while the scan ran.
    LibraryNotOpen,
    /// The scan was stopped through its progress hook (Kotlin: coroutine cancelled). Files found in the primary
    /// so far were catalogued when adding was on.
    Cancelled,
}
