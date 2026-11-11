use dms_domain::DomainError;

const QUARTER: i32 = 90;

/// One page of the new file: page [`Self::page`] of the source (counted from 0), turned clockwise by
/// [`Self::rotation`] degrees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageChange {
    pub page: usize,
    pub rotation: i32,
}

impl PageChange {
    /// # Errors
    /// Fails for a rotation that is no multiple of a quarter turn.
    pub fn new(page: usize, rotation: i32) -> Result<Self, DomainError> {
        let change = Self { page, rotation };
        change.validate()?;
        Ok(change)
    }

    /// # Errors
    /// Fails for a rotation that is no multiple of a quarter turn.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.rotation % QUARTER == 0 {
            Ok(())
        } else {
            Err(DomainError::InvalidArgument("Pages turn in quarter turns".to_owned()))
        }
    }
}
