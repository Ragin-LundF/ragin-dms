use chrono::NaiveDate;

use crate::bill::Iban;
use crate::money::Money;

/// The payment data of an e-invoice (`ZUGFeRD`, Factur-X, `XRechnung`) embedded in a PDF: exact values, no guesses.
/// Every part is optional; minimal profiles leave some out.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EInvoice {
    pub seller: Option<String>,
    pub invoice_number: Option<String>,
    pub issue_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub amount: Option<Money>,
    pub iban: Option<Iban>,
    pub bic: Option<String>,
    pub payment_reference: Option<String>,
}
