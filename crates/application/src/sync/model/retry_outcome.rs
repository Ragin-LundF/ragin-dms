use crate::sync::RetryReport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryOutcome {
    Retried(RetryReport),
    LibraryNotOpen,
}
