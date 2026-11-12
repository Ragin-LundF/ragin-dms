use std::fmt;

use chrono::{DateTime, Utc};
use dms_domain::DomainError;
use dms_domain::catalog::CatalogRevision;

/// One stored copy of the catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogSnapshot {
    pub id: CatalogSnapshotId,
    pub taken_at: DateTime<Utc>,
    pub revision: CatalogRevision,
}

/// The name of a snapshot (`20261111T120000Z-r42`). Only ASCII letters, digits and `-`, so an id from the window can
/// never point outside the snapshot folder.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CatalogSnapshotId(String);

impl CatalogSnapshotId {
    /// # Errors
    /// Fails for an empty id or one with other characters.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.is_empty() || !value.chars().all(|char| char.is_ascii_alphanumeric() || char == '-') {
            return Err(DomainError::InvalidArgument(format!("Not a snapshot: {value}")));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CatalogSnapshotId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
