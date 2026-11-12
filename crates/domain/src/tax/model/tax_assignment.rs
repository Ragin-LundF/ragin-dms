use crate::DomainError;
use crate::error::require;
use crate::money::Money;
use crate::tax::TaxArea;

/// A document's place in a tax return: the year, the area and, where it differs from the document's amount, the
/// deductible part (such as the labour share of a craftsman's bill).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaxAssignment {
    pub year: i32,
    pub area: TaxArea,
    pub deductible: Option<Money>,
}

impl TaxAssignment {
    /// # Errors
    /// Fails for a year outside 1000..=9999.
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_tax_year(self.year)
    }
}

/// # Errors
/// Fails for a year outside 1000..=9999, which no tax return has.
pub(crate) fn validate_tax_year(year: i32) -> Result<(), DomainError> {
    require((1000..=9999).contains(&year), || format!("Not a tax year: {year}"))
}
