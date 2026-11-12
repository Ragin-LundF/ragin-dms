//! The words of a query request: today, the names the library knows, the custom fields and the tax areas.

use dms_application::port::assistant::QueryRequest;
use dms_domain::bill::BillStatus;
use dms_domain::tax::TaxArea;
use dms_domain::taxonomy::TagName;
use dms_domain::{iso_date, text};

use crate::classify::prompt_texts::{listed, path_of, type_name};
use crate::prompt;

pub(crate) fn system(request: &QueryRequest) -> String {
    prompt::QUERY.fill(&[("today", &iso_date(request.today))])
}

pub(crate) fn request(request: &QueryRequest) -> String {
    let mut categories: Vec<String> =
        request.categories.iter().map(|category| path_of(category, &request.categories)).collect();
    categories.sort_by(|first, second| text::compare(first, second));
    let tags: Vec<&str> = request.tags.iter().map(TagName::value).collect();
    let fields: Vec<String> = request
        .fields
        .iter()
        .map(|field| format!("{}: {}, {}", field.key.value(), field.label, type_name(field.field_type)))
        .collect();
    let areas: Vec<String> =
        request.tax_areas.iter().map(|area| format!("{}: {}", tax_area_key(*area), tax_area_meaning(*area))).collect();
    prompt::QUERY_REQUEST.fill(&[
        ("categories", &listed(&categories)),
        ("tags", &listed(&tags)),
        ("correspondents", &listed(&request.correspondents)),
        ("fields", &listed(&fields)),
        ("taxAreas", &listed(&areas)),
        ("sentence", &request.sentence),
    ])
}

pub(crate) fn tax_area_key(area: TaxArea) -> &'static str {
    match area {
        TaxArea::Income => "income",
        TaxArea::PensionInsurance => "pensionInsurance",
        TaxArea::HouseholdServices => "householdServices",
        TaxArea::Donations => "donations",
        TaxArea::HealthCosts => "healthCosts",
        TaxArea::WorkExpenses => "workExpenses",
        TaxArea::CapitalIncome => "capitalIncome",
        TaxArea::Rental => "rental",
        TaxArea::Other => "other",
    }
}

fn tax_area_meaning(area: TaxArea) -> &'static str {
    match area {
        TaxArea::Income => "wages and salary (Anlage N)",
        TaxArea::PensionInsurance => "insurance and pension contributions (Vorsorgeaufwand)",
        TaxArea::HouseholdServices => "household services and craftsmen (§ 35a)",
        TaxArea::Donations => "donations (Sonderausgaben)",
        TaxArea::HealthCosts => "health costs (außergewöhnliche Belastungen)",
        TaxArea::WorkExpenses => "work-related expenses (Werbungskosten)",
        TaxArea::CapitalIncome => "interest and dividends (Anlage KAP)",
        TaxArea::Rental => "letting a property (Anlage V)",
        TaxArea::Other => "anything else for the tax return",
    }
}

pub(crate) fn payment_of(raw: &str) -> Option<BillStatus> {
    match text::lowercase(text::trim(raw)).as_str() {
        "open" | "unpaid" => Some(BillStatus::Open),
        "paid" => Some(BillStatus::Paid),
        "overdue" => Some(BillStatus::Overdue),
        _ => None,
    }
}
