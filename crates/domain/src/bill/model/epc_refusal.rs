use crate::field::CurrencyCode;

/// Why no EPC payment code (`GiroCode`) can be made for a bill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpcRefusal {
    /// EPC codes only carry euro amounts.
    NotEuro(CurrencyCode),
    /// The amount lies outside 0.01 to 999,999,999.99.
    AmountOutOfRange,
    /// The bill names no payee.
    PayeeMissing,
    /// The payee's name is longer than 70 characters.
    PayeeTooLong,
    /// The payment reference is longer than 140 characters.
    ReferenceTooLong,
}
