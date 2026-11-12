use chrono::NaiveDate;
use dms_domain::bill::BillStatus;
use dms_domain::tax::TaxArea;

use crate::port::assistant::FieldAnswer;

/// The filter parts the AI read from a sentence, names as it wrote them; the use case checks them against the
/// library.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TranslatedQuery {
    pub text: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub correspondent: Option<String>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub fields: Vec<FieldAnswer>,
    pub payment: Option<BillStatus>,
    pub tax_year: Option<i32>,
    pub tax_area: Option<TaxArea>,
}
