use thiserror::Error;

/// A value or request breaks a domain rule. The Rust counterpart of the `IllegalArgumentException` that the
/// Kotlin `require` checks throw; the message is the same text.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("{0}")]
    InvalidArgument(String),
}

/// Returns [`DomainError::InvalidArgument`] with `message` unless `condition` holds.
pub(crate) fn require(condition: bool, message: impl FnOnce() -> String) -> Result<(), DomainError> {
    if condition { Ok(()) } else { Err(DomainError::InvalidArgument(message())) }
}
