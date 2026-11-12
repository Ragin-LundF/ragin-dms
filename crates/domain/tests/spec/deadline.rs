use chrono::{Days, NaiveDate};
use dms_domain::catalog::Catalog;
use dms_domain::deadline::DeadlinePlanner;
use dms_domain::document::Document;
use dms_domain::field::{FieldPresets, FieldRole, FieldValue};
use dms_domain::retention::Retention;
use dms_domain::settings::CatalogSettings;
use dms_domain::task::OpenTask;
use indexmap::indexmap;

use crate::test_data::{self, Applied, date};

fn today() -> NaiveDate {
    date(2026, 3, 1)
}

fn in_days(days: u64) -> NaiveDate {
    today() + Days::new(days)
}

fn contract() -> Document {
    test_data::document("contract", "contract.pdf")
}

fn bill() -> Document {
    test_data::document("bill", "bill.pdf")
}

fn base() -> Catalog {
    test_data::catalog_with(&[contract(), bill()])
        .change_settings(CatalogSettings { field_definitions: FieldPresets::all(), ..CatalogSettings::default() })
        .applied()
}

fn with_dates(cancel_by: NaiveDate, due: NaiveDate) -> Catalog {
    base()
        .set_fields(&contract().id, indexmap! { FieldPresets::cancel_by().key => FieldValue::Date(cancel_by) })
        .applied()
        .set_fields(&bill().id, indexmap! { FieldPresets::due_date().key => FieldValue::Date(due) })
        .applied()
}

fn keep_until(date: NaiveDate) -> Retention {
    Retention { keep_until: Some(date), settled_at: None }
}

#[test]
fn deadlines_appear_within_the_lead_time_of_their_role() {
    let catalog = with_dates(in_days(30), in_days(8));

    let tasks = DeadlinePlanner::due_tasks(&catalog, today(), FieldRole::default_lead_days);

    assert_eq!(
        tasks,
        vec![OpenTask::DeadlineDue {
            document_id: contract().id,
            field_key: FieldPresets::cancel_by().key,
            date: in_days(30)
        }]
    );
}

#[test]
fn the_task_key_changes_with_the_date_so_a_dismissed_task_returns() {
    let first = DeadlinePlanner::due_tasks(&with_dates(in_days(3), in_days(1)), today(), FieldRole::default_lead_days);
    let moved = DeadlinePlanner::due_tasks(&with_dates(in_days(4), in_days(1)), today(), FieldRole::default_lead_days);

    assert_ne!(first[0].key(), moved[0].key());
    assert_eq!(first[first.len() - 1].key(), moved[moved.len() - 1].key());
}

#[test]
fn a_retention_task_appears_once_its_date_is_reached() {
    let catalog = base()
        .set_retention(&contract().id, Some(keep_until(today())))
        .applied()
        .set_retention(&bill().id, Some(keep_until(in_days(1))))
        .applied();

    let tasks = DeadlinePlanner::due_tasks(&catalog, today(), FieldRole::default_lead_days);

    assert_eq!(tasks, vec![OpenTask::RetentionDue { document_id: contract().id, date: today() }]);
}

#[test]
fn upcoming_lists_the_next_dates_in_order() {
    let catalog = with_dates(in_days(90), in_days(2))
        .set_retention(&bill().id, Some(keep_until(today() - Days::new(1))))
        .applied();

    let upcoming = DeadlinePlanner::upcoming(&catalog, today(), 10);

    assert_eq!(upcoming.iter().map(|it| it.date).collect::<Vec<_>>(), vec![in_days(2), in_days(90)]);
    assert_eq!(DeadlinePlanner::upcoming(&catalog, today(), 1).len(), 1);
}

fn bill_due(due: NaiveDate, paid: bool) -> Catalog {
    let mut fields = indexmap! {
        FieldPresets::amount().key => FieldValue::Money { amount: test_data::decimal("12.30"), currency: test_data::euro() },
        FieldPresets::due_date().key => FieldValue::Date(due),
    };
    if paid {
        fields.insert(FieldPresets::paid_on().key, FieldValue::Date(today()));
    }
    let settings = CatalogSettings {
        field_definitions: FieldPresets::all().into_iter().chain([FieldPresets::paid_on()]).collect(),
        ..CatalogSettings::default()
    };
    base().change_settings(settings).applied().set_fields(&bill().id, fields).applied()
}

#[test]
fn the_due_date_of_a_paid_bill_is_no_deadline_and_not_upcoming() {
    let catalog = bill_due(in_days(2), true);

    assert_eq!(DeadlinePlanner::due_tasks(&catalog, today(), FieldRole::default_lead_days), vec![]);
    assert_eq!(DeadlinePlanner::upcoming(&catalog, today(), 10), vec![]);
}

#[test]
fn an_unpaid_bill_keeps_its_deadline_task_until_it_is_overdue() {
    let due_soon = bill_due(in_days(2), false);
    let overdue = bill_due(today() - Days::new(1), false);

    assert_eq!(DeadlinePlanner::due_tasks(&due_soon, today(), FieldRole::default_lead_days).len(), 1);
    assert_eq!(DeadlinePlanner::due_tasks(&overdue, today(), FieldRole::default_lead_days), vec![]);
}
