use thiserror::Error;

use crate::port::storage::StorageFailure;

/// The result of a storage operation. Expected failures are values, not panics. Kotlin's
/// `StorageResult.Success(value)` is `Ok(value)`, `StorageResult.Failure(reason, detail)` is `Err(StorageError)`.
pub type StorageResult<T> = Result<T, StorageError>;

/// Why a storage operation failed, with an optional detail for logs.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{reason:?}{}", detail.as_deref().map(|detail| format!(": {detail}")).unwrap_or_default())]
pub struct StorageError {
    pub reason: StorageFailure,
    pub detail: Option<String>,
}

impl StorageError {
    #[must_use]
    pub fn new(reason: StorageFailure) -> Self {
        Self { reason, detail: None }
    }

    #[must_use]
    pub fn with_detail(reason: StorageFailure, detail: impl Into<String>) -> Self {
        Self { reason, detail: Some(detail.into()) }
    }
}
