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

/// Fails with "`what` must not be blank" for blank text.
pub(crate) fn require_not_blank(value: &str, what: &str) -> Result<(), DomainError> {
    require(!crate::text::is_blank(value), || format!("{what} must not be blank"))
}

/// Fails with "`what` must be empty or not blank" for given but blank text.
pub(crate) fn require_none_or_not_blank(value: Option<&str>, what: &str) -> Result<(), DomainError> {
    require(value.is_none_or(|value| !crate::text::is_blank(value)), || format!("{what} must be empty or not blank"))
}

/// Fails with `message` when two of `items` are equal.
pub(crate) fn require_distinct<T: Eq + std::hash::Hash>(
    items: impl IntoIterator<Item = T>,
    message: impl FnOnce() -> String,
) -> Result<(), DomainError> {
    let mut seen = std::collections::HashSet::new();
    require(items.into_iter().all(|item| seen.insert(item)), message)
}
