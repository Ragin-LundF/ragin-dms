use crate::DomainError;
use crate::contract::NoticeUnit;
use crate::error::require;

/// The notice period: `count` days, weeks or months before the term ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Notice {
    pub count: u16,
    pub unit: NoticeUnit,
}

impl Notice {
    /// # Errors
    /// Fails for a notice of zero.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(self.count > 0, || "A notice period must be at least 1".to_owned())
    }
}
