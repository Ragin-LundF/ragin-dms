use dms_domain::bill::BillStatus;
use dms_domain::query::DocumentQuery;
use dms_domain::tax::TaxArea;

/// A sentence turned into filters: the library query and the parts that are no query condition (payment state, tax
/// year and area). [`Self::not_understood`] names, as the AI wrote them, what the library does not know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryTranslation {
    pub query: DocumentQuery,
    pub payment: Option<BillStatus>,
    pub tax_year: Option<i32>,
    pub tax_area: Option<TaxArea>,
    pub not_understood: Vec<String>,
    pub model: String,
}
