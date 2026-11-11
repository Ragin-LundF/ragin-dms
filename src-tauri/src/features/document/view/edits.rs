//! What the user typed into the quick look and its details, checked and turned into one catalog change (Kotlin
//! `QuickLookViewModel.saveFact/saveMetadata/addTag/rename/moveToFolder` and `DetailsController.save*`). A value
//! that does not fit is refused with the message the user reads; nothing is saved then.

use dms_application::catalog::CatalogEdit;
use dms_application::document::DetailEdit;
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentMetadata};
use dms_domain::field::{FieldDefinition, FieldKey, FieldPresets, FieldType, FieldValues};
use dms_domain::library::RelativePath;
use dms_domain::retention::Retention;
use dms_domain::rule::{Correspondent, MatchingRule, MatchingRuleId, RuleActions, RuleCondition, RuleOrigin};
use dms_domain::settings::CatalogSettings;
use dms_domain::taxonomy::{Category, CategoryId, TagName};
use dms_domain::text;

use super::quick_look::{amount_field, due_field};
use super::texts::{DATE_PROBLEM, name_problem, parse_date};
use crate::features::document::dto::{QuickFact, TypedField};

/// One change of the catalog, through `EditCatalog` or `EditDocumentDetails`.
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentEdit {
    Catalog(CatalogEdit),
    Details(DetailEdit),
}

/// What saving a typed fact does: save a change, end the edit without one, or keep the edit open with the
/// `message` (none when the field disappeared meanwhile).
#[derive(Debug, Clone, PartialEq)]
pub enum FactPlan {
    Save(DocumentEdit),
    Nothing,
    Refused(Option<String>),
}

pub fn plan_fact(catalog: &Catalog, document: &Document, fact: QuickFact, typed: &str) -> FactPlan {
    let metadata = &document.metadata;
    match fact {
        QuickFact::Title => plan_metadata(document, Some(typed), metadata.correspondent.as_deref(), None),
        QuickFact::Correspondent => plan_metadata(document, metadata.title.as_deref(), Some(typed), None),
        QuickFact::Date => {
            plan_metadata(document, metadata.title.as_deref(), metadata.correspondent.as_deref(), Some(typed))
        }
        QuickFact::Amount => plan_field(catalog, document, amount_field(catalog), typed),
        QuickFact::Due => plan_field(catalog, document, due_field(catalog), typed),
        QuickFact::Tag => plan_tag(document, typed),
    }
}

/// Title, correspondent and date saved together; `typed_date` `None` keeps the current date.
fn plan_metadata(
    document: &Document,
    title: Option<&str>,
    correspondent: Option<&str>,
    typed_date: Option<&str>,
) -> FactPlan {
    let document_date = match typed_date {
        None => document.metadata.document_date,
        Some(raw) if text::is_blank(raw) => None,
        Some(raw) => match parse_date(raw) {
            Some(date) => Some(date),
            None => return FactPlan::Refused(Some(DATE_PROBLEM.to_owned())),
        },
    };
    let cleaned = |value: Option<&str>| value.map(text::trim).filter(|value| !value.is_empty()).map(str::to_owned);
    let metadata = DocumentMetadata { title: cleaned(title), document_date, correspondent: cleaned(correspondent) };
    FactPlan::Save(DocumentEdit::Catalog(CatalogEdit::UpdateMetadata { document_id: document.id.clone(), metadata }))
}

fn plan_field(catalog: &Catalog, document: &Document, field: Option<&FieldDefinition>, typed: &str) -> FactPlan {
    let Some(definition) = field else {
        return FactPlan::Refused(None);
    };
    let mut fields = document.fields.clone();
    if text::is_blank(typed) {
        fields.shift_remove(&definition.key);
    } else {
        let currency = catalog.settings().layout.default_currency.as_ref();
        let Some(value) = FieldValues::parse(definition.field_type, typed, currency) else {
            return FactPlan::Refused(Some(mismatch(definition)));
        };
        fields.insert(definition.key.clone(), value);
    }
    FactPlan::Save(DocumentEdit::Details(DetailEdit::SetFields { document_id: document.id.clone(), fields }))
}

fn plan_tag(document: &Document, typed: &str) -> FactPlan {
    if text::is_blank(typed) {
        return FactPlan::Nothing;
    }
    let Some(tag) = TagName::parse(typed) else {
        return FactPlan::Refused(Some(format!("A tag has at most {} characters", TagName::MAX_LENGTH)));
    };
    let tags = std::iter::once(tag).collect();
    FactPlan::Save(DocumentEdit::Catalog(CatalogEdit::AddTags { document_id: document.id.clone(), tags }))
}

/// All typed field values saved together: blank ones are removed, values of fields the library no longer defines
/// are kept, and one that does not fit refuses the whole save with a message naming the field.
pub fn plan_fields(catalog: &Catalog, document: &Document, typed: &[TypedField]) -> Result<DetailEdit, String> {
    let settings = catalog.settings();
    let currency = settings.layout.default_currency.as_ref();
    let mut fields = document.fields.clone();
    fields.retain(|key, _| settings.field(key).is_none());
    for entry in typed {
        let Some(definition) = FieldKey::new(entry.key.as_str()).ok().and_then(|key| settings.field(&key).cloned())
        else {
            continue;
        };
        if text::is_blank(&entry.value) {
            continue;
        }
        let value =
            FieldValues::parse(definition.field_type, &entry.value, currency).ok_or_else(|| mismatch(&definition))?;
        fields.insert(definition.key, value);
    }
    Ok(DetailEdit::SetFields { document_id: document.id.clone(), fields })
}

/// `"{label}": {hint}` for a value that does not fit its field.
fn mismatch(definition: &FieldDefinition) -> String {
    let hint = match definition.field_type {
        FieldType::Text => "write some text",
        FieldType::Number => "write a number",
        FieldType::Money => "write an amount, such as 12.50 EUR",
        FieldType::Date => "write a date such as 2026-12-31",
        FieldType::Boolean => "write yes or no",
    };
    format!("\"{}\": {hint}", definition.label)
}

/// The note, unless it equals the saved one once trailing spaces are gone.
pub fn plan_note(document: &Document, note: &str) -> Option<DetailEdit> {
    if text::trim_end(note) == document.note.as_deref().unwrap_or_default() {
        return None;
    }
    Some(DetailEdit::SetNote { document_id: document.id.clone(), note: Some(note.to_owned()) })
}

/// The keep-until date (`YYYY-MM-DD`), or none when empty; a retention marked as done stays done.
pub fn plan_keep_until(document: &Document, typed: &str) -> Result<DetailEdit, &'static str> {
    let keep_until = if text::is_blank(typed) { None } else { Some(parse_date(typed).ok_or(DATE_PROBLEM)?) };
    let retention = Retention { keep_until, ..document.retention.clone().unwrap_or_default() };
    let retention = Some(retention).filter(|retention| !retention.is_empty());
    Ok(DetailEdit::SetRetention { document_id: document.id.clone(), retention })
}

/// How many years documents of the category are kept; empty removes the period.
pub fn plan_category_years(category_id: CategoryId, typed: &str) -> Result<DetailEdit, &'static str> {
    const PROBLEM: &str = "Write a number of years from 1 to 100";
    let trimmed = text::trim(typed);
    let years = if trimmed.is_empty() {
        None
    } else {
        let years = trimmed.parse::<i64>().map_err(|_| PROBLEM)?;
        let years = u32::try_from(years).ok().filter(|years| (1..=Category::MAX_RETENTION_YEARS).contains(years));
        Some(years.ok_or(PROBLEM)?)
    };
    Ok(DetailEdit::SetRetentionYears { category_id, years })
}

/// The new path of a rename: same folder, the trimmed name, which the file name policy must allow.
pub fn rename_target(document: &Document, typed: &str) -> Result<RelativePath, String> {
    if let Some(problem) = name_problem(typed) {
        return Err(problem.to_owned());
    }
    let parent = document.path.parent().unwrap_or_else(RelativePath::root);
    parent.resolve(text::trim(typed)).map_err(|error| error.to_string())
}

/// The new path of "Move…": the typed folder (`/` or nothing for the top level) with the same file name.
pub fn move_target(document: &Document, typed: &str) -> Result<RelativePath, String> {
    let folder = text::trim(typed).trim_matches(['/', '\\']);
    let problem = if folder.is_empty() { None } else { folder.split('/').find_map(name_problem) };
    match (RelativePath::parse(folder), problem) {
        (_, Some(problem)) => Err(problem.to_owned()),
        (None, None) => Err("Not a valid folder".to_owned()),
        (Some(parent), None) => parent.resolve(document.path.file_name()).map_err(|error| error.to_string()),
    }
}

/// A manual rule: documents of this correspondent get its category and tags proposed.
pub fn rule_from(document: &Document, id: MatchingRuleId) -> Result<MatchingRule, &'static str> {
    const PROBLEM: &str = "Give the document a correspondent and a category or tags first";
    let Some(correspondent) = &document.metadata.correspondent else {
        return Err(PROBLEM);
    };
    if document.category_id.is_none() && document.tags.is_empty() {
        return Err(PROBLEM);
    }
    let actions = RuleActions {
        category_id: document.category_id.clone(),
        tags: document.tags.clone(),
        ..RuleActions::default()
    };
    let conditions = vec![RuleCondition::CorrespondentIs { name: correspondent.clone() }];
    MatchingRule::new(id, format!("Rule for {correspondent}"), conditions, actions, RuleOrigin::Manual)
        .map_err(|_| PROBLEM)
}

/// The settings with `rule`, and its correspondent as a known one unless one with that name (ignoring case) exists.
pub fn settings_with_rule(settings: &CatalogSettings, rule: &MatchingRule, correspondent: &str) -> CatalogSettings {
    let candidate = Correspondent { name: correspondent.to_owned(), aliases: Vec::new() };
    let known = settings.correspondents.iter().any(|existing| existing.key() == candidate.key());
    let with_name = if known { settings.clone() } else { settings.with_correspondent(candidate) };
    with_name.with_rule(rule.clone())
}

/// The settings with every standard field the library does not define yet.
pub fn settings_with_standard_fields(settings: &CatalogSettings) -> CatalogSettings {
    FieldPresets::all().into_iter().fold(settings.clone(), |current, preset| {
        if current.field(&preset.key).is_none() { current.with_field(preset) } else { current }
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use dms_domain::field::FieldValue;

    use super::*;
    use crate::features::document::view::test_catalog::*;

    fn saved(plan: FactPlan) -> DocumentEdit {
        match plan {
            FactPlan::Save(edit) => edit,
            other => panic!("expected a save, got {other:?}"),
        }
    }

    fn metadata_of(edit: DocumentEdit) -> DocumentMetadata {
        match edit {
            DocumentEdit::Catalog(CatalogEdit::UpdateMetadata { metadata, .. }) => metadata,
            other => panic!("expected metadata, got {other:?}"),
        }
    }

    fn named_document() -> Document {
        let mut letter = document(1, "Home/letter.pdf");
        letter.metadata = DocumentMetadata {
            title: Some("Letter".to_owned()),
            document_date: Some(day("2026-02-01")),
            correspondent: Some("City Power".to_owned()),
        };
        letter
    }

    #[test]
    fn metadata_facts_are_saved_together_trimmed_and_empty_removes_them() {
        let letter = named_document();
        let catalog = catalog(vec![letter.clone()], vec![], vec![], CatalogSettings::default());

        let title = metadata_of(saved(plan_fact(&catalog, &letter, QuickFact::Title, "  Power bill ")));
        assert_eq!(title, DocumentMetadata { title: Some("Power bill".to_owned()), ..letter.metadata.clone() });
        let no_correspondent = metadata_of(saved(plan_fact(&catalog, &letter, QuickFact::Correspondent, "   ")));
        assert_eq!(no_correspondent.correspondent, None);
        assert_eq!(no_correspondent.document_date, Some(day("2026-02-01")));
        let no_date = metadata_of(saved(plan_fact(&catalog, &letter, QuickFact::Date, "")));
        assert_eq!(no_date.document_date, None);
        let date = metadata_of(saved(plan_fact(&catalog, &letter, QuickFact::Date, " 2026-03-04 ")));
        assert_eq!(date.document_date, Some(day("2026-03-04")));
    }

    #[test]
    fn a_date_that_is_no_iso_date_is_refused() {
        let letter = named_document();
        let catalog = catalog(vec![letter.clone()], vec![], vec![], CatalogSettings::default());
        assert_eq!(
            plan_fact(&catalog, &letter, QuickFact::Date, "04.03.2026"),
            FactPlan::Refused(Some(DATE_PROBLEM.to_owned()))
        );
    }

    #[test]
    fn amount_and_due_are_parsed_by_their_field_and_empty_removes_only_that_value() {
        let settings = standard_settings();
        let mut bill = document(1, "bill.pdf");
        field_value(&mut bill, "due", "2026-04-01", &settings);
        let catalog = catalog(vec![bill.clone()], vec![], vec![], settings);

        let DocumentEdit::Details(DetailEdit::SetFields { fields, .. }) =
            saved(plan_fact(&catalog, &bill, QuickFact::Amount, "12,50"))
        else {
            panic!("expected fields");
        };
        let amount = fields.get(&FieldKey::new("amount").expect("key")).expect("amount saved");
        assert_eq!(FieldValues::format(amount), "12.50 EUR");
        assert_eq!(fields.len(), 2, "the due date stays");

        let DocumentEdit::Details(DetailEdit::SetFields { fields, .. }) =
            saved(plan_fact(&catalog, &bill, QuickFact::Due, " "))
        else {
            panic!("expected fields");
        };
        assert!(fields.is_empty());
        assert_eq!(
            plan_fact(&catalog, &bill, QuickFact::Due, "soon"),
            FactPlan::Refused(Some("\"Due date\": write a date such as 2026-12-31".to_owned()))
        );
    }

    #[test]
    fn amount_without_a_money_field_keeps_the_edit_open_silently() {
        let bill = document(1, "bill.pdf");
        let catalog = catalog(vec![bill.clone()], vec![], vec![], CatalogSettings::default());
        assert_eq!(plan_fact(&catalog, &bill, QuickFact::Amount, "12"), FactPlan::Refused(None));
    }

    #[test]
    fn tags_blank_adds_nothing_and_too_long_is_refused() {
        let bill = document(1, "bill.pdf");
        let catalog = catalog(vec![bill.clone()], vec![], vec![], CatalogSettings::default());
        assert_eq!(plan_fact(&catalog, &bill, QuickFact::Tag, "  "), FactPlan::Nothing);
        assert_eq!(
            plan_fact(&catalog, &bill, QuickFact::Tag, &"x".repeat(61)),
            FactPlan::Refused(Some("A tag has at most 60 characters".to_owned()))
        );
        let DocumentEdit::Catalog(CatalogEdit::AddTags { tags, .. }) =
            saved(plan_fact(&catalog, &bill, QuickFact::Tag, " tax  2025 "))
        else {
            panic!("expected tags");
        };
        assert_eq!(tags.iter().map(TagName::value).collect::<Vec<_>>(), ["tax 2025"]);
    }

    #[test]
    fn fields_save_together_keep_undefined_values_and_refuse_on_any_mismatch() {
        let settings = standard_settings();
        let mut bill = document(1, "bill.pdf");
        bill.fields.insert(FieldKey::new("old").expect("key"), FieldValue::Flag(true));
        field_value(&mut bill, "amount", "5 EUR", &settings);
        let catalog = catalog(vec![bill.clone()], vec![], vec![], settings);
        let typed = |pairs: &[(&str, &str)]| -> Vec<TypedField> {
            pairs.iter().map(|(key, value)| TypedField { key: (*key).to_owned(), value: (*value).to_owned() }).collect()
        };

        let DetailEdit::SetFields { fields, .. } =
            plan_fields(&catalog, &bill, &typed(&[("amount", ""), ("due", "2026-05-01"), ("unknown", "x")]))
                .expect("saved")
        else {
            panic!("expected fields");
        };
        let keys: Vec<&str> = fields.keys().map(FieldKey::value).collect();
        assert_eq!(keys, ["old", "due"]);

        let refused = plan_fields(&catalog, &bill, &typed(&[("due", "2026-05-01"), ("amount", "lots")]));
        assert_eq!(refused, Err("\"Amount\": write an amount, such as 12.50 EUR".to_owned()));
    }

    #[test]
    fn note_unchanged_after_trailing_spaces_saves_nothing() {
        let mut bill = document(1, "bill.pdf");
        bill.note = Some("Paid".to_owned());
        assert_eq!(plan_note(&bill, "Paid  \n"), None);
        assert!(plan_note(&bill, "Paid late").is_some());
        assert!(plan_note(&document(2, "b.pdf"), "").is_none(), "no note and nothing typed");
    }

    #[test]
    fn keep_until_keeps_a_done_retention_done_and_empty_removes_the_date() {
        let mut bill = document(1, "bill.pdf");
        let settled_at = Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).single();
        bill.retention = Some(Retention { keep_until: None, settled_at });

        let Ok(DetailEdit::SetRetention { retention, .. }) = plan_keep_until(&bill, "2030-12-31") else {
            panic!("expected retention");
        };
        assert_eq!(retention, Some(Retention { keep_until: Some(day("2030-12-31")), settled_at }));
        let Ok(DetailEdit::SetRetention { retention, .. }) = plan_keep_until(&document(2, "b.pdf"), "") else {
            panic!("expected retention");
        };
        assert_eq!(retention, None, "an empty retention is stored as none");
        assert_eq!(plan_keep_until(&bill, "someday"), Err(DATE_PROBLEM));
    }

    #[test]
    fn category_years_must_be_one_to_hundred() {
        let years = |typed: &str| match plan_category_years(category_id("c1"), typed) {
            Ok(DetailEdit::SetRetentionYears { years, .. }) => Ok(years),
            Ok(other) => panic!("unexpected {other:?}"),
            Err(problem) => Err(problem),
        };
        assert_eq!(years(" 10 "), Ok(Some(10)));
        assert_eq!(years(""), Ok(None));
        for wrong in ["0", "101", "-3", "ten"] {
            assert_eq!(years(wrong), Err("Write a number of years from 1 to 100"), "{wrong}");
        }
    }

    #[test]
    fn rename_stays_in_the_folder_trims_and_checks_the_name_policy() {
        let letter = document(1, "Home/letter.pdf");
        assert_eq!(
            rename_target(&letter, "  bill.PDF ").map(|path| path.value().to_owned()),
            Ok("Home/bill.PDF".to_owned())
        );
        assert_eq!(rename_target(&letter, "con.txt"), Err("Windows reserves this name".to_owned()));
        assert_eq!(rename_target(&document(2, "top.pdf"), "renamed"), Ok(RelativePath::of("renamed").expect("path")));
    }

    #[test]
    fn move_takes_a_folder_or_the_top_level_and_names_the_first_problem() {
        let letter = document(1, "Home/letter.pdf");
        let target = |typed: &str| move_target(&letter, typed).map(|path| path.value().to_owned());
        assert_eq!(target(" /Letters/2024/ "), Ok("Letters/2024/letter.pdf".to_owned()));
        assert_eq!(target("/"), Ok("letter.pdf".to_owned()));
        assert_eq!(target(""), Ok("letter.pdf".to_owned()));
        assert_eq!(target("Letters/a:b"), Err("The name contains one of < > : \" / \\ | ? *".to_owned()));
        assert_eq!(target("Letters//2024"), Err("The name is empty".to_owned()));
        assert_eq!(target("Letters/../x"), Err("The name must not start with a dot or a space".to_owned()));
    }

    #[test]
    fn a_rule_needs_a_correspondent_and_a_category_or_tags() {
        let rule_id = || MatchingRuleId::new("r1").expect("rule id");
        let mut letter = document(1, "letter.pdf");
        let problem = Err("Give the document a correspondent and a category or tags first");
        assert_eq!(rule_from(&letter, rule_id()).map(|rule| rule.name), problem);
        letter.metadata.correspondent = Some("City Power".to_owned());
        assert_eq!(rule_from(&letter, rule_id()).map(|rule| rule.name), problem);
        letter.tags = std::iter::once(tag("invoice")).collect();

        let rule = rule_from(&letter, rule_id()).expect("rule");
        assert_eq!(rule.name, "Rule for City Power");
        assert_eq!(rule.conditions, [RuleCondition::CorrespondentIs { name: "City Power".to_owned() }]);
        assert_eq!(rule.origin, RuleOrigin::Manual);
        assert_eq!(rule.actions.tags, letter.tags);
    }

    #[test]
    fn rule_settings_add_the_correspondent_only_when_no_one_has_that_name_ignoring_case() {
        let mut letter = with_tags(document(1, "letter.pdf"), &["invoice"]);
        letter.metadata.correspondent = Some("City Power".to_owned());
        let rule = rule_from(&letter, MatchingRuleId::new("r1").expect("id")).expect("rule");

        let added = settings_with_rule(&CatalogSettings::default(), &rule, "City Power");
        assert_eq!((added.correspondents.len(), added.rules.len()), (1, 1));
        let known = CatalogSettings::default()
            .with_correspondent(Correspondent { name: "city power".to_owned(), aliases: Vec::new() });
        let kept = settings_with_rule(&known, &rule, "City Power");
        assert_eq!(kept.correspondents[0].name, "city power");
        assert_eq!(kept.correspondents.len(), 1);
    }

    #[test]
    fn standard_fields_are_added_only_where_missing() {
        let mut settings = CatalogSettings::default();
        let custom_amount = FieldDefinition { label: "Total".to_owned(), ..FieldPresets::amount() };
        settings.field_definitions = vec![custom_amount];
        let added = settings_with_standard_fields(&settings);
        let labels: Vec<&str> = added.field_definitions.iter().map(|field| field.label.as_str()).collect();
        assert_eq!(labels, ["Total", "Due date", "Cancel by", "Contract end", "Warranty until"]);
    }
}
