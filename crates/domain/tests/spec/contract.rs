//! Contracts: cancel-by dates, proposals, the dashboard tasks and the totals.

use chrono::NaiveDate;
use dms_domain::catalog::Catalog;
use dms_domain::contract::{
    Contract, ContractPlanner, ContractPrice, ContractProposal, ContractProposer, ContractTerms, ContractTotals,
    Notice, NoticeUnit, PricePoint, ProposalReason,
};
use dms_domain::deadline::DeadlinePlanner;
use dms_domain::document::Document;
use dms_domain::field::{CurrencyCode, FieldPresets, FieldRole, FieldValue};
use dms_domain::money::BillingPeriod;
use dms_domain::settings::CatalogSettings;
use dms_domain::task::OpenTask;
use dms_domain::taxonomy::Category;
use indexmap::{IndexMap, indexmap};

use crate::test_data::{self, Applied, category_id, date, decimal, document_id, euro};

fn today() -> NaiveDate {
    date(2026, 11, 11)
}

fn months(count: u16) -> Notice {
    Notice { count, unit: NoticeUnit::Months }
}

fn terms(term_end: NaiveDate, renews_by_months: Option<u16>, notice: Option<Notice>) -> ContractTerms {
    ContractTerms { term_end: Some(term_end), renews_by_months, notice, cancellable_any_time: false }
}

#[test]
fn a_yearly_contract_with_automatic_renewal_rolls_its_cancel_by_date_into_the_next_term() {
    // Started 1 March 2025 for a year, renews by 12 months, 3 months notice: the first chance (28 Nov 2025) passed.
    let contract = terms(date(2026, 2, 28), Some(12), Some(months(3)));

    assert_eq!(contract.next_cancel_by(today()), Some(date(2026, 11, 30)));
    assert_eq!(contract.next_term_end(today()), Some(date(2027, 2, 28)));
}

#[test]
fn a_one_month_notice_to_the_end_of_the_year_ends_on_the_last_day_of_november() {
    let contract = terms(date(2026, 12, 31), Some(12), Some(months(1)));

    assert_eq!(contract.next_cancel_by(today()), Some(date(2026, 11, 30)));
    assert_eq!(contract.next_cancel_by(date(2026, 12, 1)), Some(date(2027, 11, 30)));
}

#[test]
fn after_the_minimum_term_a_contract_renewing_monthly_can_be_cancelled_every_month() {
    // § 309 BGB: 24 months until 30 June 2026, then monthly with one month notice.
    let contract = terms(date(2026, 6, 30), Some(1), Some(months(1)));

    assert_eq!(contract.next_cancel_by(today()), Some(date(2026, 11, 30)));
    assert_eq!(contract.next_term_end(today()), Some(date(2026, 12, 31)));
}

#[test]
fn the_cancel_by_day_itself_still_counts_and_weeks_and_days_count_back_from_the_term_end() {
    let in_weeks = terms(date(2026, 12, 23), None, Some(Notice { count: 6, unit: NoticeUnit::Weeks }));
    let in_days = terms(date(2026, 11, 25), None, Some(Notice { count: 14, unit: NoticeUnit::Days }));

    assert_eq!(in_weeks.next_cancel_by(today()), Some(date(2026, 11, 11)));
    assert_eq!(in_days.next_cancel_by(today()), Some(date(2026, 11, 11)));
}

#[test]
fn no_cancel_by_date_without_a_term_end_when_cancellable_any_time_or_after_the_last_term() {
    let any_time = ContractTerms { cancellable_any_time: true, ..terms(date(2026, 12, 31), Some(12), Some(months(1))) };
    let over = terms(date(2026, 10, 31), None, Some(months(1)));
    let no_notice = terms(date(2026, 12, 31), None, None);

    assert_eq!(ContractTerms::default().next_cancel_by(today()), None);
    assert_eq!(any_time.next_cancel_by(today()), None);
    assert_eq!(over.next_cancel_by(today()), None);
    assert_eq!(no_notice.next_cancel_by(today()), Some(date(2026, 12, 31)));
}

fn price(amount: &str, period: BillingPeriod) -> ContractPrice {
    ContractPrice { amount: decimal(amount), currency: euro(), period }
}

fn point(valid_from: NaiveDate, amount: &str) -> PricePoint {
    PricePoint { valid_from, price: price(amount, BillingPeriod::Monthly), document_id: None }
}

fn contract_with(id: &str, terms: ContractTerms, documents: &[&str]) -> Contract {
    let mut contract = test_data::contract(id, "Internet");
    contract.correspondent = Some("Net Provider".to_owned());
    contract.terms = terms;
    contract.document_ids = documents.iter().map(|id| document_id(id)).collect();
    contract
}

fn dated(id: &str, correspondent: &str, fields: IndexMap<dms_domain::field::FieldKey, FieldValue>) -> Document {
    let mut document = test_data::document(id, &format!("{id}.pdf"));
    document.metadata.correspondent = Some(correspondent.to_owned());
    document.fields = fields;
    document
}

fn catalog_of(documents: &[Document]) -> Catalog {
    test_data::catalog_with(documents)
        .change_settings(CatalogSettings { field_definitions: FieldPresets::all(), ..CatalogSettings::default() })
        .applied()
}

fn cancel_by(on: NaiveDate) -> IndexMap<dms_domain::field::FieldKey, FieldValue> {
    indexmap! { FieldPresets::cancel_by().key => FieldValue::Date(on) }
}

#[test]
fn a_contract_shows_its_cancel_by_task_from_thirty_days_ahead() {
    let catalog = catalog_of(&[]);
    let due = contract_with("c-1", terms(date(2026, 12, 11), Some(12), None), &[]);
    let later = contract_with("c-2", terms(date(2026, 12, 12), Some(12), None), &[]);
    let catalog = catalog.save_contract(due).applied().save_contract(later).applied();

    let tasks = ContractPlanner::due_tasks(&catalog, today());

    assert_eq!(
        tasks,
        vec![OpenTask::ContractCancelBy {
            contract_id: dms_domain::contract::ContractId::new("c-1").expect("id"),
            date: date(2026, 12, 11)
        }]
    );
}

#[test]
fn an_ended_contract_has_no_cancel_by_task() {
    let mut ended = contract_with("c-1", terms(date(2026, 11, 30), Some(12), None), &[]);
    ended.ended_on = Some(date(2027, 11, 30));
    let catalog = catalog_of(&[]).save_contract(ended).applied();

    assert_eq!(ContractPlanner::due_tasks(&catalog, today()), Vec::<OpenTask>::new());
}

#[test]
fn the_contract_task_replaces_the_cancel_by_field_task_of_its_documents_only_while_it_plans_a_date() {
    let letter = dated("letter", "Net Provider", cancel_by(date(2026, 11, 30)));
    let base = catalog_of(&[letter]).change_settings(CatalogSettings {
        dismissed_contract_proposals: vec!["net provider".to_owned()],
        ..catalog_of(&[]).settings().clone()
    });
    let base = base.applied();
    let planning =
        base.save_contract(contract_with("c-1", terms(date(2026, 11, 30), Some(12), None), &["letter"])).applied();
    let without_terms = base.save_contract(contract_with("c-1", ContractTerms::default(), &["letter"])).applied();

    let field_tasks = |catalog: &Catalog| {
        DeadlinePlanner::due_tasks(catalog, today(), FieldRole::default_lead_days)
            .into_iter()
            .filter(|task| matches!(task, OpenTask::DeadlineDue { .. }))
            .count()
    };

    assert_eq!(field_tasks(&base), 1);
    assert_eq!(field_tasks(&planning), 0);
    assert_eq!(field_tasks(&without_terms), 1);
}

#[test]
fn a_document_with_a_cancel_by_date_proposes_a_contract_with_that_document() {
    let catalog = catalog_of(&[
        dated("letter", "Gym Club", cancel_by(date(2027, 3, 31))),
        dated("other", " gym club ", IndexMap::new()),
    ]);

    assert_eq!(
        ContractProposer::proposals(&catalog, today()),
        vec![ContractProposal {
            correspondent: "Gym Club".to_owned(),
            document_ids: vec![document_id("letter")],
            reason: ProposalReason::DeadlineField,
        }]
    );
}

fn categorized(id: &str, correspondent: &str, category: &str) -> Document {
    let mut document = dated(id, correspondent, IndexMap::new());
    document.category_id = Some(category_id(category));
    document
}

fn with_categories(documents: &[Document]) -> Catalog {
    let categories = [("ins", "Versicherungen"), ("abo", "Abos"), ("lab", "Labor")];
    let base = categories.iter().fold(catalog_of(&[]), |catalog, (id, name)| {
        catalog.add_category(Category::new(category_id(id), *name, None, None).expect("category")).applied()
    });
    documents.iter().fold(base, |catalog, document| catalog.add_document(document.clone()).applied())
}

#[test]
fn two_documents_in_a_contract_category_propose_a_contract_but_a_similar_word_does_not() {
    let catalog = with_categories(&[
        categorized("i-1", "Car Insurer", "ins"),
        categorized("i-2", "Car Insurer", "ins"),
        categorized("s-1", "Streaming Box", "abo"),
        categorized("s-2", "Streaming Box", "abo"),
        categorized("l-1", "Medical Lab", "lab"),
        categorized("l-2", "Medical Lab", "lab"),
        categorized("single", "Lonely Insurer", "ins"),
    ]);

    let proposals = ContractProposer::proposals(&catalog, today());

    let names: Vec<(&str, ProposalReason)> =
        proposals.iter().map(|proposal| (proposal.correspondent.as_str(), proposal.reason)).collect();
    assert_eq!(
        names,
        vec![("Car Insurer", ProposalReason::ContractCategory), ("Streaming Box", ProposalReason::ContractCategory)]
    );
}

fn bill(index: u32, on: NaiveDate) -> Document {
    let money = FieldValue::Money { amount: decimal("49.99"), currency: euro() };
    let mut document = dated(
        &format!("bill-{index}"),
        "Power Works",
        indexmap! { FieldPresets::amount().key => money, FieldPresets::due_date().key => FieldValue::Date(on) },
    );
    document.metadata.document_date = Some(on);
    document
}

#[test]
fn monthly_bills_of_one_correspondent_propose_a_contract() {
    let catalog = catalog_of(&[bill(1, date(2026, 8, 1)), bill(2, date(2026, 9, 1)), bill(3, date(2026, 10, 1))]);

    let proposals = ContractProposer::proposals(&catalog, today());

    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].reason, ProposalReason::RecurringBills);
    assert_eq!(proposals[0].document_ids.len(), 3);
}

#[test]
fn dismissed_correspondents_and_those_with_a_contract_get_no_proposal() {
    let documents = [
        dated("a", "Gym Club", cancel_by(date(2027, 3, 31))),
        dated("b", "Net Provider", cancel_by(date(2027, 3, 31))),
        dated("c", "Phone Co", cancel_by(date(2027, 3, 31))),
    ];
    let catalog = catalog_of(&documents);
    let settings =
        CatalogSettings { dismissed_contract_proposals: vec!["GYM CLUB".to_owned()], ..catalog.settings().clone() };
    let mut by_document = contract_with("c-2", ContractTerms::default(), &["c"]);
    by_document.correspondent = None;
    let catalog = catalog
        .change_settings(settings)
        .applied()
        .save_contract(contract_with("c-1", ContractTerms::default(), &[]))
        .applied()
        .save_contract(by_document)
        .applied();

    assert_eq!(ContractProposer::proposals(&catalog, today()), Vec::<ContractProposal>::new());
}

#[test]
fn a_new_price_point_becomes_the_current_price_once_it_is_valid() {
    let contract = test_data::contract("c-1", "Internet");

    let announced = contract.with_price_point(point(date(2027, 1, 1), "49.99"), today());
    let valid = announced.with_price_point(point(date(2026, 1, 1), "44.99"), today());

    assert_eq!(announced.price, None);
    assert_eq!(valid.price, Some(price("44.99", BillingPeriod::Monthly)));
    assert_eq!(
        valid.price_history.iter().map(|point| point.valid_from).collect::<Vec<_>>(),
        vec![date(2026, 1, 1), date(2027, 1, 1)]
    );
}

#[test]
fn totals_add_running_contracts_per_month_and_year_and_compare_with_last_year() {
    let mut internet = test_data::contract("c-1", "Internet");
    internet = internet
        .with_price_point(point(date(2024, 1, 1), "39.99"), today())
        .with_price_point(point(date(2026, 1, 1), "44.99"), today());
    let mut insurance = test_data::contract("c-2", "Insurance");
    insurance.price = Some(price("120.00", BillingPeriod::Yearly));
    let mut ended = test_data::contract("c-3", "Old phone");
    ended.price = Some(price("10.00", BillingPeriod::Monthly));
    ended.ended_on = Some(date(2026, 10, 1));
    let mut dollars = test_data::contract("c-4", "Cloud");
    dollars.price = Some(ContractPrice {
        amount: decimal("5.00"),
        currency: CurrencyCode::new("USD").expect("currency"),
        period: BillingPeriod::Monthly,
    });

    let totals = ContractTotals::of([&internet, &insurance, &ended, &dollars], today());

    let euros = |totals: &dms_domain::money::MoneyTotals| totals.get(&euro()).map(ToString::to_string);
    assert_eq!(euros(&totals.monthly), Some("54.99".to_owned()));
    assert_eq!(euros(&totals.yearly), Some("659.88".to_owned()));
    assert_eq!(euros(&totals.monthly_change), Some("5.00".to_owned()));
    assert_eq!(totals.monthly.amounts().len(), 2);
    assert_eq!(totals.price_increases, 1);
}
