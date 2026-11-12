use chrono::Days;
use chrono::NaiveDate;
use dms_domain::catalog::Catalog;
use dms_domain::document::Document;
use dms_domain::expectation::{
    ExpectationMode, ExpectationOverride, ExpectationPlanner, ExpectationSnoozes, ExpectedDocument, Rhythm,
    RhythmDetector,
};
use dms_domain::settings::CatalogSettings;
use dms_domain::taxonomy::Category;

use crate::test_data::{self, Applied, category_id, date};

fn letter(index: usize, correspondent: &str, on: NaiveDate, category: Option<&str>) -> Document {
    let mut document = test_data::document(&format!("doc-{index}"), &format!("letters/{index}.pdf"));
    document.metadata.correspondent = Some(correspondent.to_owned());
    document.metadata.document_date = Some(on);
    document.category_id = category.map(category_id);
    document
}

/// A catalog with a "Salary" category and one document per `(correspondent, date)`.
fn catalog_of(letters: &[(&str, NaiveDate)], category: Option<&str>) -> Catalog {
    let salary = Category::new(category_id("salary"), "Salary", None, None).expect("valid category");
    let base = Catalog::empty(test_data::library_id()).add_category(salary).applied();
    letters.iter().enumerate().fold(base, |catalog, (index, (correspondent, on))| {
        catalog.add_document(letter(index, correspondent, *on, category)).applied()
    })
}

fn monthly_slips(months: &[u32]) -> Vec<(&'static str, NaiveDate)> {
    months.iter().map(|month| ("Employer Ltd", date(2026, *month, 3))).collect()
}

fn with_override(catalog: &Catalog, mode: ExpectationMode, category: Option<&str>) -> Catalog {
    let expectation =
        ExpectationOverride { correspondent: "employer ltd".to_owned(), category_id: category.map(category_id), mode };
    let settings = CatalogSettings { expectations: vec![expectation], ..catalog.settings().clone() };
    catalog.change_settings(settings).applied()
}

#[test]
fn monthly_salary_slips_with_one_gap_in_the_past_produce_exactly_one_missing_document() {
    // January to September, March missing; October's slip is late on 2026-10-20.
    let catalog = catalog_of(&monthly_slips(&[1, 2, 4, 5, 6, 7, 8, 9]), Some("salary"));

    let missing = RhythmDetector::missing(&catalog, date(2026, 10, 20));

    assert_eq!(
        missing,
        vec![ExpectedDocument {
            correspondent: "Employer Ltd".to_owned(),
            category_id: Some(category_id("salary")),
            expected_on: date(2026, 10, 4),
            rhythm: Rhythm::Monthly,
            first_seen_year: 2026,
        }]
    );
}

#[test]
fn a_document_is_missing_only_after_the_tolerance() {
    let catalog = catalog_of(&monthly_slips(&[1, 2, 3, 4, 5, 6]), None);
    let expected_on = RhythmDetector::expected(&catalog, date(2026, 6, 10))[0].expected_on;

    assert_eq!(RhythmDetector::missing(&catalog, plus(expected_on, 10)), Vec::new(), "10 days of tolerance");
    assert_eq!(RhythmDetector::missing(&catalog, plus(expected_on, 11)).len(), 1);
}

fn plus(day: NaiveDate, days: u64) -> NaiveDate {
    day.checked_add_days(Days::new(days)).expect("valid date")
}

#[test]
fn irregular_senders_and_too_few_documents_expect_nothing() {
    let irregular = [
        ("Shop", date(2026, 1, 1)),
        ("Shop", date(2026, 1, 20)),
        ("Shop", date(2026, 4, 2)),
        ("Shop", date(2026, 4, 9)),
        ("Shop", date(2026, 9, 30)),
    ];
    let two = [("Insurer", date(2025, 1, 15)), ("Insurer", date(2026, 1, 15))];

    assert_eq!(RhythmDetector::expected(&catalog_of(&irregular, None), date(2026, 10, 1)), Vec::new());
    assert_eq!(RhythmDetector::expected(&catalog_of(&two, None), date(2026, 3, 1)), Vec::new());
}

#[test]
fn the_same_day_twice_counts_once() {
    let letters = [("Insurer", date(2024, 2, 1)), ("Insurer", date(2024, 2, 1)), ("Insurer", date(2025, 2, 1))];

    assert_eq!(RhythmDetector::expected(&catalog_of(&letters, None), date(2025, 3, 1)), Vec::new());
}

#[test]
fn seventy_percent_of_the_gaps_must_lie_in_the_band() {
    // Gaps 31, 28, 31, 90: three of four (75 %) monthly.
    let mostly = monthly_with_extra(&[1, 2, 3, 4], date(2026, 7, 3));
    // Gaps 31, 28, 92: two of three (67 %) monthly.
    let too_few = monthly_with_extra(&[1, 2, 3], date(2026, 6, 3));

    assert_eq!(RhythmDetector::expected(&catalog_of(&mostly, None), date(2026, 7, 10)).len(), 1);
    assert_eq!(RhythmDetector::expected(&catalog_of(&too_few, None), date(2026, 6, 10)), Vec::new());
}

fn monthly_with_extra(months: &[u32], extra: NaiveDate) -> Vec<(&'static str, NaiveDate)> {
    let mut letters = monthly_slips(months);
    letters.push(("Employer Ltd", extra));
    letters
}

#[test]
fn the_rhythm_follows_the_median_gap_and_the_next_date_is_the_last_plus_that_gap() {
    let yearly = [
        ("Building Society", date(2021, 1, 20)),
        ("Building Society", date(2022, 1, 18)),
        ("Building Society", date(2023, 1, 25)),
    ];
    let quarterly =
        [("Water Works", date(2026, 1, 10)), ("Water Works", date(2026, 4, 10)), ("Water Works", date(2026, 7, 10))];

    let yearly = RhythmDetector::expected(&catalog_of(&yearly, None), date(2023, 2, 1));
    let quarterly = RhythmDetector::expected(&catalog_of(&quarterly, None), date(2026, 7, 11));

    assert_eq!(
        (yearly[0].rhythm, yearly[0].expected_on, yearly[0].first_seen_year),
        (Rhythm::Yearly, date(2024, 1, 27), 2021)
    );
    assert_eq!((quarterly[0].rhythm, quarterly[0].expected_on), (Rhythm::Quarterly, date(2026, 10, 8)));
}

#[test]
fn after_two_missed_periods_nothing_is_expected_unless_the_user_keeps_it() {
    let catalog = catalog_of(&monthly_slips(&[1, 2, 3, 4]), Some("salary"));
    // Last slip 2026-04-03, median 31 days: the second period ends 2026-06-04, plus 10 days of tolerance.
    let still = date(2026, 6, 14);
    let stopped = date(2026, 6, 15);

    assert_eq!(RhythmDetector::missing(&catalog, still).len(), 1);
    assert_eq!(RhythmDetector::missing(&catalog, stopped), Vec::new());
    let kept = with_override(&catalog, ExpectationMode::Keep, Some("salary"));
    assert_eq!(RhythmDetector::missing(&kept, stopped).len(), 1);
}

#[test]
fn an_off_override_suppresses_the_sender_of_that_category_only() {
    let catalog = catalog_of(&monthly_slips(&[1, 2, 3, 4]), Some("salary"));
    let today = date(2026, 5, 20);

    assert_eq!(
        RhythmDetector::missing(&with_override(&catalog, ExpectationMode::Off, Some("salary")), today),
        Vec::new()
    );
    assert_eq!(RhythmDetector::missing(&with_override(&catalog, ExpectationMode::Off, None), today).len(), 1);
}

#[test]
fn senders_are_grouped_ignoring_case_and_named_like_their_latest_document() {
    let letters =
        [("employer ltd", date(2026, 1, 3)), ("EMPLOYER LTD", date(2026, 2, 3)), ("Employer Ltd", date(2026, 3, 3))];

    let expected = RhythmDetector::expected(&catalog_of(&letters, None), date(2026, 3, 10));

    assert_eq!(expected.len(), 1);
    assert_eq!(expected[0].correspondent, "Employer Ltd");
}

#[test]
fn the_planner_makes_one_task_per_missing_document_and_hides_snoozed_ones_until_their_day() {
    let catalog = catalog_of(&monthly_slips(&[1, 2, 4, 5, 6, 7, 8, 9]), Some("salary"));
    let today = date(2026, 10, 20);
    let tasks = ExpectationPlanner::due_tasks(&catalog, today, &ExpectationSnoozes::default());
    let key = tasks[0].key();
    let snoozes = ExpectationSnoozes { snoozed_until: indexmap::IndexMap::from([(key.clone(), plus(today, 14))]) };

    assert_eq!(tasks.len(), 1);
    assert_eq!(key.value(), "expected:employer ltd:salary:2026-10-04");
    assert_eq!(ExpectationPlanner::due_tasks(&catalog, plus(today, 13), &snoozes), Vec::new());
    assert_eq!(ExpectationPlanner::due_tasks(&catalog, plus(today, 14), &snoozes).len(), 1);
}

#[test]
fn irregular_senders_get_no_task() {
    let letters = [
        ("Random Shop", date(2026, 1, 5)),
        ("Random Shop", date(2026, 1, 9)),
        ("Random Shop", date(2026, 4, 30)),
        ("Random Shop", date(2026, 5, 2)),
    ];

    assert_eq!(
        ExpectationPlanner::due_tasks(&catalog_of(&letters, None), date(2026, 10, 1), &ExpectationSnoozes::default()),
        Vec::new()
    );
}
