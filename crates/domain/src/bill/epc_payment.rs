use crate::bill::{EpcRefusal, Iban};
use crate::field::Decimal;
use crate::money::Money;
use crate::text;

/// A SEPA credit transfer as an EPC QR code ("`GiroCode`", EPC069-12, version 002, UTF-8): payee, IBAN, an optional
/// BIC, a euro amount and the payment reference as unstructured remittance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpcPayment {
    payee: String,
    iban: Iban,
    bic: Option<String>,
    amount: Decimal,
    reference: Option<String>,
}

const MAX_PAYEE: usize = 70;
const MAX_REFERENCE: usize = 140;

impl EpcPayment {
    /// A BIC that is not 8 or 11 letters and digits is left out; version 002 does not need one.
    ///
    /// # Errors
    /// Refuses other currencies than euro, amounts outside 0.01 to 999,999,999.99, a blank or too long payee and a
    /// too long reference.
    pub fn new(
        payee: &str,
        iban: Iban,
        bic: Option<&str>,
        amount: &Money,
        reference: Option<&str>,
    ) -> Result<Self, EpcRefusal> {
        if amount.currency.value() != "EUR" {
            return Err(EpcRefusal::NotEuro(amount.currency.clone()));
        }
        let cents = amount.amount.divided_rounded(1, 2);
        let in_range = |bound: &str| Decimal::parse(bound).map(|bound| cents.compare_value(&bound));
        if in_range("0.01") == Some(std::cmp::Ordering::Less)
            || in_range("999999999.99") == Some(std::cmp::Ordering::Greater)
        {
            return Err(EpcRefusal::AmountOutOfRange);
        }
        let payee = text::trim(payee);
        if payee.is_empty() {
            return Err(EpcRefusal::PayeeMissing);
        }
        if payee.chars().count() > MAX_PAYEE {
            return Err(EpcRefusal::PayeeTooLong);
        }
        let reference = reference.map(text::trim).filter(|reference| !reference.is_empty());
        if reference.is_some_and(|reference| reference.chars().count() > MAX_REFERENCE) {
            return Err(EpcRefusal::ReferenceTooLong);
        }
        let bic = bic.map(text::trim).filter(|bic| is_bic(bic)).map(str::to_uppercase);
        Ok(Self { payee: payee.to_owned(), iban, bic, amount: cents, reference: reference.map(str::to_owned) })
    }

    /// The text the QR code holds, lines separated by `\n`; empty optional lines at the end are left out.
    #[must_use]
    pub fn payload(&self) -> String {
        let lines = [
            "BCD".to_owned(),
            "002".to_owned(),
            "1".to_owned(),
            "SCT".to_owned(),
            self.bic.clone().unwrap_or_default(),
            self.payee.clone(),
            self.iban.value().to_owned(),
            format!("EUR{}", without_trailing_zeros(&self.amount.to_plain_string())),
            String::new(),
            String::new(),
            self.reference.clone().unwrap_or_default(),
        ];
        let used = lines.iter().rposition(|line| !line.is_empty()).map_or(0, |last| last + 1);
        lines[..used].join("\n")
    }
}

fn is_bic(bic: &str) -> bool {
    matches!(bic.len(), 8 | 11) && bic.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

/// `12.30` → `12.3`, `1.00` → `1`.
fn without_trailing_zeros(plain: &str) -> &str {
    if plain.contains('.') { plain.trim_end_matches('0').trim_end_matches('.') } else { plain }
}
