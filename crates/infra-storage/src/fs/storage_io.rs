use std::io;

use dms_application::port::storage::{StorageError, StorageFailure};
use dms_domain::DomainError;

/// An I/O exception of the Kotlin version: `IO_ERROR` with the message as detail.
pub fn io_error(error: &io::Error) -> StorageError {
    StorageError::with_detail(StorageFailure::IoError, error.to_string())
}

pub fn failure(reason: StorageFailure) -> StorageError {
    StorageError::new(reason)
}

/// A file whose content breaks a domain rule; the Kotlin version caught the `IllegalArgumentException`.
pub fn format_error(error: &DomainError) -> StorageError {
    StorageError::with_detail(StorageFailure::UnsupportedFormat, error.to_string())
}

/// `?` for I/O results inside storage operations.
pub trait OrIoError<T> {
    /// # Errors
    /// Turns the I/O error into `IO_ERROR`.
    fn or_io(self) -> Result<T, StorageError>;
}

impl<T> OrIoError<T> for io::Result<T> {
    fn or_io(self) -> Result<T, StorageError> {
        self.map_err(|error| io_error(&error))
    }
}
