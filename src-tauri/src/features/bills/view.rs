//! The presenters of the bills screen: pure functions from domain types to DTOs.

use chrono::{Datelike, NaiveDate};
use dms_domain::bill::{BillPlanner, BillStatus, EpcPayment, Iban};
use dms_domain::catalog::Catalog;
use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::{ClassificationSource, Document, DocumentId};
use dms_domain::expectation::{ExpectedDocument, Rhythm};
use dms_domain::field::{FieldPresets, FieldValue, FieldValues};
use dms_domain::money::{Money, MoneyTotals};
use dms_domain::task::OpenTask;
use dms_domain::{iso_date, text};

use super::dto::{
    BillDocumentDto, BillPaymentCodeDto, BillRowDto, BillSourceDto, BillStatusDto, BillsFilterDto, BillsTotalDto,
    BillsViewDto, ExpectedDocumentDto,
};
use crate::messages;

/// How many expected documents the dashboard card lists.
const EXPECTED_LIMIT: usize = 5;
/// How many of a sender's documents a request letter refers to.
const LETTER_SOURCES: usize = 3;

/// One bill with what every list needs.
struct Bill<'c> {
    document: &'c Document,
    status: BillStatus,
    due: NaiveDate,
    amount: Option<Money>,
}

pub fn bills_view(catalog: &Catalog, today: NaiveDate, filter: BillsFilterDto) -> BillsViewDto {
    let older_ids: Vec<DocumentId> = BillPlanner::never_marked(catalog).into_iter().map(|(id, _)| id).collect();
    let (older, mut tracked): (Vec<Bill<'_>>, Vec<Bill<'_>>) =
        bills(catalog, today).into_iter().partition(|bill| older_ids.contains(&bill.document.id));
    tracked.sort_by_key(|bill| bill.due);
    let unpaid: Vec<&Bill<'_>> = tracked.iter().filter(|bill| bill.status != BillStatus::Paid).collect();
    let month_end = end_of_month(today);
    let paid_this_month =
        tracked.iter().filter(|bill| paid_on(bill.document).is_some_and(|day| same_month(day, today)));
    let shown = tracked.iter().filter(|bill| match filter {
        BillsFilterDto::Open => bill.status != BillStatus::Paid,
        BillsFilterDto::Paid => bill.status == BillStatus::Paid,
        BillsFilterDto::All => true,
    });
    BillsViewDto {
        open: total(unpaid.iter().copied()),
        overdue: total(unpaid.iter().copied().filter(|bill| bill.status == BillStatus::Overdue)),
        paid_this_month: total(paid_this_month),
        due_this_month: total(unpaid.iter().copied().filter(|bill| bill.due <= month_end)),
        month: today.format("%B").to_string(),
        rows: shown.map(|bill| row(bill, catalog, today)).collect(),
        older: older.iter().map(|bill| row(bill, catalog, today)).collect(),
        tracking_started: catalog.settings().bills_tracked_since.is_some(),
    }
}

/// The payment dialog of `document`, and the payment the QR code holds; `None` with the reason in the DTO.
pub fn payment_code(
    catalog: &Catalog,
    document: &Document,
    today: NaiveDate,
) -> (BillPaymentCodeDto, Option<EpcPayment>) {
    let iban = text_field(document, &FieldPresets::iban().key).and_then(|raw| Iban::parse(raw).ok());
    let reference = text_field(document, &FieldPresets::payment_reference().key);
    let amount = amount_of(document);
    let payee = document.metadata.correspondent.clone().unwrap_or_default();
    let payment = match (&iban, &amount) {
        (Some(iban), Some(amount)) => EpcPayment::new(&payee, iban.clone(), None, amount, reference).map_err(Some),
        _ => Err(None),
    };
    let status = BillStatus::of(document, catalog.settings(), today);
    let dto = BillPaymentCodeDto {
        document_id: document.id.value().to_owned(),
        title: title_of(document),
        svg: None,
        refusal: payment.as_ref().err().map(|refusal| messages::payment_code_refusal(refusal.as_ref())),
        payee,
        iban: iban.as_ref().map(Iban::grouped),
        amount: amount.as_ref().map(money_text).unwrap_or_default(),
        reference: reference.map(str::to_owned),
        due: BillStatus::due_date(document, catalog.settings()).map(iso_date).unwrap_or_default(),
        source: source_of(document),
        paid: status == Some(BillStatus::Paid),
    };
    (dto, payment.ok())
}

pub fn bill_document(catalog: &Catalog, document: &Document, today: NaiveDate) -> BillDocumentDto {
    let status = BillStatus::of(document, catalog.settings(), today);
    BillDocumentDto {
        is_bill: status.is_some(),
        status: status.map(status_dto),
        paid_on: paid_on(document).map(iso_date),
        can_read_e_invoice: document.kind() == dms_domain::document::DocumentKind::Pdf,
    }
}

/// The first "expected but missing" tasks for the dashboard card.
pub fn expected_documents(tasks: &[OpenTask], catalog: &Catalog) -> Vec<ExpectedDocumentDto> {
    tasks
        .iter()
        .filter_map(|task| match task {
            OpenTask::ExpectedDocumentMissing(expected) => Some((task.key(), expected)),
            _ => None,
        })
        .take(EXPECTED_LIMIT)
        .map(|(key, expected)| ExpectedDocumentDto {
            task_key: key.value().to_owned(),
            title: expected_title(expected, catalog),
            correspondent: expected.correspondent.clone(),
            rhythm: rhythm_words(expected),
            expected_on: iso_date(expected.expected_on),
            document_ids: latest_of(expected, catalog),
        })
        .collect()
}

/// The category of an expected document, or a general name.
pub fn expected_title(expected: &ExpectedDocument, catalog: &Catalog) -> String {
    expected
        .category_id
        .as_ref()
        .and_then(|id| catalog.categories().get(id))
        .map_or_else(|| "Regular document".to_owned(), |category| category.name.clone())
}

/// The rhythm in words, such as "Arrived every January since 2021".
pub fn rhythm_words(expected: &ExpectedDocument) -> String {
    let every = match expected.rhythm {
        Rhythm::Monthly => "month".to_owned(),
        Rhythm::Quarterly => "quarter".to_owned(),
        Rhythm::HalfYearly => "six months".to_owned(),
        Rhythm::Yearly => expected.expected_on.format("%B").to_string(),
    };
    format!("Arrived every {every} since {}", expected.first_seen_year)
}

/// The amount of a bill as text, for the dashboard.
pub fn amount_text(document: &Document) -> Option<String> {
    amount_of(document).as_ref().map(money_text)
}

fn bills(catalog: &Catalog, today: NaiveDate) -> Vec<Bill<'_>> {
    let settings = catalog.settings();
    catalog
        .documents()
        .values()
        .filter_map(|document| {
            Some(Bill {
                status: BillStatus::of(document, settings, today)?,
                due: BillStatus::due_date(document, settings)?,
                amount: amount_of(document),
                document,
            })
        })
        .collect()
}

fn row(bill: &Bill<'_>, catalog: &Catalog, today: NaiveDate) -> BillRowDto {
    let (code, _) = payment_code(catalog, bill.document, today);
    BillRowDto {
        document_id: bill.document.id.value().to_owned(),
        title: title_of(bill.document),
        payee: bill.document.metadata.correspondent.clone().unwrap_or_default(),
        due: iso_date(bill.due),
        due_in_days: i32::try_from((bill.due - today).num_days()).unwrap_or(i32::MIN),
        status: status_dto(bill.status),
        paid_on: paid_on(bill.document).map(iso_date),
        amount: bill.amount.as_ref().map(money_text).unwrap_or_default(),
        source: source_of(bill.document),
        qr_refusal: code.refusal,
    }
}

fn total<'b>(bills: impl Iterator<Item = &'b Bill<'b>>) -> BillsTotalDto {
    let mut count = 0u32;
    let totals = bills.fold(MoneyTotals::default(), |totals, bill| {
        count += 1;
        match &bill.amount {
            Some(amount) => totals.plus(amount),
            None => totals,
        }
    });
    let amount = totals.amounts().iter().map(money_text).collect::<Vec<_>>().join(" + ");
    BillsTotalDto { amount, count }
}

fn money_text(money: &Money) -> String {
    FieldValues::format(&FieldValue::Money { amount: money.amount.clone(), currency: money.currency.clone() })
}

fn amount_of(document: &Document) -> Option<Money> {
    document.fields.get(&FieldPresets::amount().key).and_then(Money::of_field)
}

fn paid_on(document: &Document) -> Option<NaiveDate> {
    match document.fields.get(&FieldPresets::paid_on().key) {
        Some(FieldValue::Date(day)) => Some(*day),
        _ => None,
    }
}

fn text_field<'d>(document: &'d Document, key: &dms_domain::field::FieldKey) -> Option<&'d str> {
    match document.fields.get(key) {
        Some(FieldValue::Text(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn title_of(document: &Document) -> String {
    document.metadata.title.clone().unwrap_or_else(|| document.path.file_name().to_owned())
}

fn source_of(document: &Document) -> BillSourceDto {
    let Some(record) = &document.classification else { return BillSourceDto::Typed };
    if record.model.as_deref().is_some_and(|model| model.ends_with(ClassificationSuggestion::E_INVOICE_MODEL)) {
        return BillSourceDto::EInvoice;
    }
    match record.source {
        ClassificationSource::Ai => BillSourceDto::Ai,
        ClassificationSource::Rule => BillSourceDto::Rule,
        ClassificationSource::Manual => BillSourceDto::Typed,
    }
}

fn status_dto(status: BillStatus) -> BillStatusDto {
    match status {
        BillStatus::Open => BillStatusDto::Open,
        BillStatus::Overdue => BillStatusDto::Overdue,
        BillStatus::Paid => BillStatusDto::Paid,
    }
}

fn same_month(day: NaiveDate, today: NaiveDate) -> bool {
    day.year() == today.year() && day.month() == today.month()
}

fn end_of_month(today: NaiveDate) -> NaiveDate {
    let (year, month) = if today.month() == 12 { (today.year() + 1, 1) } else { (today.year(), today.month() + 1) };
    NaiveDate::from_ymd_opt(year, month, 1).and_then(|first| first.pred_opt()).unwrap_or(today)
}

/// The sender's latest documents of that category, newest first.
fn latest_of(expected: &ExpectedDocument, catalog: &Catalog) -> Vec<String> {
    let sender = text::lowercase(text::trim(&expected.correspondent));
    let mut documents: Vec<&Document> = catalog
        .documents()
        .values()
        .filter(|document| {
            document.category_id == expected.category_id
                && document
                    .metadata
                    .correspondent
                    .as_deref()
                    .is_some_and(|name| text::lowercase(text::trim(name)) == sender)
        })
        .collect();
    documents.sort_by_key(|document| std::cmp::Reverse(document.metadata.document_date));
    documents.into_iter().take(LETTER_SOURCES).map(|document| document.id.value().to_owned()).collect()
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Days};
    use dms_domain::catalog::{CatalogUpdate, LibraryId};
    use dms_domain::document::{ClassificationRecord, ContentHash};
    use dms_domain::field::{CurrencyCode, Decimal, FieldKey};
    use dms_domain::library::RelativePath;
    use dms_domain::settings::CatalogSettings;

    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 11).expect("date")
    }

    struct BillSpec {
        name: &'static str,
        due: NaiveDate,
        paid_on: Option<NaiveDate>,
        currency: &'static str,
        iban: Option<&'static str>,
        model: Option<&'static str>,
    }

    fn bill(name: &'static str, due: NaiveDate) -> BillSpec {
        BillSpec { name, due, paid_on: None, currency: "EUR", iban: Some("DE89370400440532013000"), model: None }
    }

    fn catalog(bills: &[BillSpec], tracked_since: Option<NaiveDate>) -> Catalog {
        let settings = CatalogSettings {
            field_definitions: vec![
                FieldPresets::amount(),
                FieldPresets::due_date(),
                FieldPresets::paid_on(),
                FieldPresets::iban(),
                FieldPresets::payment_reference(),
            ],
            bills_tracked_since: tracked_since,
            ..CatalogSettings::default()
        };
        let empty = Catalog::empty(LibraryId::new("library").expect("id"));
        let start = applied(empty.change_settings(settings).expect("settings"));
        bills.iter().enumerate().fold(start, |catalog, (index, spec)| {
            let mut document = Document::new(
                DocumentId::new(spec.name).expect("id"),
                RelativePath::of(&format!("{}.pdf", spec.name)).expect("path"),
                ContentHash::new(format!("{index:064x}")).expect("hash"),
                1,
                DateTime::from_timestamp(0, 0).expect("time"),
            )
            .expect("document");
            document.metadata.correspondent = Some("Sample Energy".to_owned());
            let field = |key: FieldKey, value: FieldValue| (key, value);
            document.fields.extend([
                field(
                    FieldPresets::amount().key,
                    FieldValue::Money {
                        amount: Decimal::parse("86.40").expect("decimal"),
                        currency: CurrencyCode::new(spec.currency).expect("currency"),
                    },
                ),
                field(FieldPresets::due_date().key, FieldValue::Date(spec.due)),
            ]);
            if let Some(day) = spec.paid_on {
                document.fields.insert(FieldPresets::paid_on().key, FieldValue::Date(day));
            }
            if let Some(iban) = spec.iban {
                document.fields.insert(FieldPresets::iban().key, FieldValue::Text(iban.to_owned()));
            }
            document.classification = spec.model.map(|model| ClassificationRecord {
                source: ClassificationSource::Ai,
                model: Some(model.to_owned()),
                confirmed_at: DateTime::from_timestamp(0, 0).expect("time"),
            });
            applied(catalog.add_document(document).expect("valid"))
        })
    }

    fn applied(update: CatalogUpdate) -> Catalog {
        match update {
            CatalogUpdate::Applied(catalog) => catalog,
            CatalogUpdate::Rejected(reason) => panic!("rejected: {reason:?}"),
        }
    }

    fn names(rows: &[BillRowDto]) -> Vec<&str> {
        rows.iter().map(|row| row.document_id.as_str()).collect()
    }

    #[test]
    fn the_list_shows_bills_by_due_date_with_totals_and_older_bills_apart() {
        let catalog = catalog(
            &[
                bill("later", today() + Days::new(20)),
                bill("late", today() - Days::new(2)),
                BillSpec { paid_on: Some(today()), ..bill("paid", today() - Days::new(1)) },
                bill("ancient", today() - Days::new(90)),
            ],
            Some(today() - Days::new(30)),
        );

        let open = bills_view(&catalog, today(), BillsFilterDto::Open);
        let paid = bills_view(&catalog, today(), BillsFilterDto::Paid);

        assert_eq!(names(&open.rows), ["late", "later"]);
        assert_eq!(names(&open.older), ["ancient"]);
        assert_eq!(names(&paid.rows), ["paid"]);
        assert_eq!(open.open, BillsTotalDto { amount: "172.80 EUR".to_owned(), count: 2 });
        assert_eq!(open.overdue, BillsTotalDto { amount: "86.40 EUR".to_owned(), count: 1 });
        assert_eq!(open.paid_this_month.count, 1);
        assert_eq!(open.due_this_month.count, 1, "the overdue one; the other is due next month");
        assert_eq!(open.rows[0].due_in_days, -2);
        assert_eq!(open.rows[0].status, BillStatusDto::Overdue);
        assert_eq!(open.month, "November");
        assert!(open.tracking_started);
    }

    #[test]
    fn the_data_source_names_e_invoice_ai_and_typed_values() {
        let catalog = catalog(
            &[
                BillSpec { model: Some("e-invoice"), ..bill("exact", today()) },
                BillSpec { model: Some("qwen + e-invoice"), ..bill("mixed", today()) },
                BillSpec { model: Some("qwen"), ..bill("guessed", today()) },
                bill("typed", today()),
            ],
            None,
        );

        let rows = bills_view(&catalog, today(), BillsFilterDto::All).rows;

        let sources: Vec<BillSourceDto> = rows.iter().map(|row| row.source).collect();
        assert_eq!(
            sources,
            [BillSourceDto::EInvoice, BillSourceDto::EInvoice, BillSourceDto::Ai, BillSourceDto::Typed]
        );
        assert!(!bills_view(&catalog, today(), BillsFilterDto::Open).tracking_started);
    }

    #[test]
    fn a_payment_code_needs_a_valid_iban_and_a_euro_amount() {
        let catalog = catalog(
            &[
                bill("ok", today()),
                BillSpec { iban: None, ..bill("no-iban", today()) },
                BillSpec { currency: "CHF", ..bill("francs", today()) },
            ],
            None,
        );
        let code =
            |name: &str| payment_code(&catalog, &catalog.documents()[&DocumentId::new(name).expect("id")], today());

        let (ok, payment) = code("ok");

        assert_eq!(ok.iban.as_deref(), Some("DE89 3704 0044 0532 0130 00"));
        assert_eq!(ok.refusal, None);
        assert!(payment.expect("a payment").payload().contains("EUR86.4"));
        assert_eq!(code("no-iban").0.refusal.as_deref(), Some("The bill has no valid IBAN; add it in the details"));
        assert_eq!(code("francs").0.refusal.as_deref(), Some("Payment codes only work for euro amounts, not CHF"));
        assert!(code("francs").1.is_none());
    }

    #[test]
    fn the_rhythm_is_told_in_words() {
        let expected = |rhythm| ExpectedDocument {
            correspondent: "Employer Ltd".to_owned(),
            category_id: None,
            expected_on: NaiveDate::from_ymd_opt(2027, 1, 20).expect("date"),
            rhythm,
            first_seen_year: 2021,
        };

        assert_eq!(rhythm_words(&expected(Rhythm::Yearly)), "Arrived every January since 2021");
        assert_eq!(rhythm_words(&expected(Rhythm::Monthly)), "Arrived every month since 2021");
        assert_eq!(rhythm_words(&expected(Rhythm::HalfYearly)), "Arrived every six months since 2021");
    }
}
