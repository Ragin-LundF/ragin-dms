use dms_application::classification::RuleSuggestions;
use dms_domain::catalog::{Catalog, CatalogRevision};
use dms_domain::classification::{ClassificationSuggestion, SuggestedCategory, SuggestedFolder, SuggestedTag};
use dms_domain::document::Document;
use dms_domain::field::{Decimal, FieldKey, FieldValue};
use dms_domain::library::RelativePath;
use dms_domain::rule::{MatchingRule, MatchingRuleId, RuleActions, RuleCondition, RuleOrigin, RuleResult, RuleTarget};
use dms_domain::settings::CatalogSettings;
use dms_domain::taxonomy::Category;
use indexmap::{IndexMap, IndexSet};

use crate::support::TestLibrary;
use crate::support::test_data::{category_id, library_id, path, tag};

struct Fixture {
    bills: Category,
    document: Document,
    filed: Document,
    bill_rule: MatchingRule,
    catalog: Catalog,
    ai_suggestion: ClassificationSuggestion,
}

fn amount() -> FieldKey {
    FieldKey::new("amount").expect("valid key")
}

fn number(text: &str) -> FieldValue {
    FieldValue::Number(Decimal::parse(text).expect("valid number"))
}

fn fixture() -> Fixture {
    let bills = Category::new(category_id("bills"), "Bills", None, None).expect("valid category");
    let letters = Category::new(category_id("letters"), "Letters", None, None).expect("valid category");
    let document = TestLibrary::document("doc-1", "inbox/scan.pdf", "a");
    let filed = TestLibrary::document("doc-2", "home/energy/old.pdf", "b");
    let bill_rule = MatchingRule {
        id: MatchingRuleId::new("rule-bills").expect("valid id"),
        name: "Energy bills".to_owned(),
        conditions: vec![RuleCondition::CorrespondentIs { name: "Sample Energy".to_owned() }],
        actions: RuleActions { category_id: Some(bills.id.clone()), ..RuleActions::default() },
        origin: RuleOrigin::Manual,
        enabled: true,
    };
    let catalog = Catalog::new(
        library_id("library"),
        CatalogRevision::INITIAL,
        IndexMap::from([(document.id.clone(), document.clone()), (filed.id.clone(), filed.clone())]),
        IndexMap::from([(bills.id.clone(), bills.clone()), (letters.id.clone(), letters.clone())]),
        IndexSet::from([tag("Energy")]),
        IndexSet::new(),
        CatalogSettings { rules: vec![bill_rule.clone()], ..CatalogSettings::default() },
    )
    .expect("valid catalog");
    let ai_suggestion = ClassificationSuggestion {
        title: Some("Bill".to_owned()),
        category: Some(SuggestedCategory::Existing(letters.id.clone())),
        tags: vec![SuggestedTag { name: tag("Paper"), is_new: true }],
        correspondent: Some("AI Correspondent".to_owned()),
        target_folder: Some(SuggestedFolder { path: path("home/letters"), is_new: true }),
        target_reason: Some("Looks like a letter".to_owned()),
        fields: IndexMap::from([(amount(), number("1"))]),
        ..ClassificationSuggestion::empty(document.id.clone(), "sample-model")
    };
    Fixture { bills, document, filed, bill_rule, catalog, ai_suggestion }
}

fn folder_rule(raw: &str) -> RuleResult {
    RuleResult { target_folder: Some(path(raw)), ..RuleResult::default() }
}

#[test]
fn rules_that_decided_nothing_leave_the_suggestion_as_it_is() {
    let f = fixture();

    let result = RuleSuggestions::over(f.ai_suggestion.clone(), &RuleResult::default(), &f.catalog);

    assert_eq!(result, f.ai_suggestion);
}

#[test]
fn a_suggestion_of_the_rules_alone_carries_only_their_values_and_names_them_as_its_model() {
    let f = fixture();
    let rules = RuleResult {
        correspondent: Some("Sample Energy".to_owned()),
        category_id: Some(f.bills.id.clone()),
        sources: IndexMap::from([(RuleTarget::Category, f.bill_rule.id.clone())]),
        ..RuleResult::default()
    };

    let suggestion = RuleSuggestions::of(&f.document, &rules, &f.catalog);

    assert_eq!(
        suggestion,
        ClassificationSuggestion {
            category: Some(SuggestedCategory::Existing(f.bills.id.clone())),
            correspondent: Some("Sample Energy".to_owned()),
            rule_notes: IndexMap::from([(RuleTarget::Category, "Energy bills".to_owned())]),
            ..ClassificationSuggestion::empty(f.document.id.clone(), ClassificationSuggestion::RULES_MODEL)
        }
    );
    assert!(suggestion.by_rules());
}

#[test]
fn the_values_the_rules_decided_replace_the_ones_of_the_ai() {
    let f = fixture();
    let rules = RuleResult {
        correspondent: Some("Sample Energy".to_owned()),
        category_id: Some(f.bills.id.clone()),
        fields: IndexMap::from([(amount(), number("10"))]),
        ..RuleResult::default()
    };

    let result = RuleSuggestions::over(f.ai_suggestion.clone(), &rules, &f.catalog);

    assert_eq!(result.category, Some(SuggestedCategory::Existing(f.bills.id.clone())));
    assert_eq!(result.correspondent.as_deref(), Some("Sample Energy"));
    assert_eq!(result.fields, IndexMap::from([(amount(), number("10"))]));
    assert_eq!(result.title.as_deref(), Some("Bill"));
    assert_eq!(result.model, "sample-model");
}

#[test]
fn what_the_rules_left_open_keeps_the_values_of_the_ai() {
    let f = fixture();
    let due = FieldKey::new("due").expect("valid key");
    let rules = RuleResult { fields: IndexMap::from([(due.clone(), FieldValue::Flag(true))]), ..RuleResult::default() };

    let result = RuleSuggestions::over(f.ai_suggestion.clone(), &rules, &f.catalog);

    assert_eq!(result.category, f.ai_suggestion.category);
    assert_eq!(result.correspondent.as_deref(), Some("AI Correspondent"));
    assert_eq!(result.target_folder, f.ai_suggestion.target_folder);
    assert_eq!(result.target_reason.as_deref(), Some("Looks like a letter"));
    assert_eq!(result.fields, IndexMap::from([(amount(), number("1")), (due, FieldValue::Flag(true))]));
}

#[test]
fn a_category_the_catalog_no_longer_has_is_ignored_together_with_its_rule_note() {
    let f = fixture();
    let rules = RuleResult {
        category_id: Some(category_id("removed")),
        correspondent: Some("Sample Energy".to_owned()),
        sources: IndexMap::from([
            (RuleTarget::Category, f.bill_rule.id.clone()),
            (RuleTarget::Tags, f.bill_rule.id.clone()),
        ]),
        tags: IndexSet::from([tag("Energy")]),
        ..RuleResult::default()
    };

    let result = RuleSuggestions::over(f.ai_suggestion.clone(), &rules, &f.catalog);

    assert_eq!(result.category, f.ai_suggestion.category);
    assert_eq!(result.rule_notes, IndexMap::from([(RuleTarget::Tags, "Energy bills".to_owned())]));
}

#[test]
fn rule_tags_come_first_are_united_with_the_ai_tags_and_marked_new_only_when_the_catalog_lacks_them() {
    let f = fixture();
    let rules = RuleResult { tags: IndexSet::from([tag("energy"), tag("Meter")]), ..RuleResult::default() };
    let suggestion = ClassificationSuggestion {
        tags: vec![
            SuggestedTag { name: tag("ENERGY"), is_new: true },
            SuggestedTag { name: tag("Paper"), is_new: true },
        ],
        ..f.ai_suggestion.clone()
    };

    let result = RuleSuggestions::over(suggestion, &rules, &f.catalog);

    assert_eq!(
        result.tags.iter().map(|it| (it.name.value(), it.is_new)).collect::<Vec<_>>(),
        [("energy", false), ("Meter", true), ("Paper", true)]
    );
}

#[test]
fn a_rule_folder_replaces_the_ai_folder_and_drops_the_ai_reason() {
    let f = fixture();

    let result = RuleSuggestions::over(f.ai_suggestion.clone(), &folder_rule("home/energy"), &f.catalog);

    assert_eq!(result.target_folder, Some(SuggestedFolder { path: path("home/energy"), is_new: false }));
    assert_eq!(result.target_reason, None);
}

#[test]
fn a_rule_folder_without_documents_is_marked_new() {
    let f = fixture();

    let result = RuleSuggestions::over(f.ai_suggestion.clone(), &folder_rule("home/water"), &f.catalog);

    assert_eq!(result.target_folder, Some(SuggestedFolder { path: path("home/water"), is_new: true }));
}

#[test]
fn a_folder_holding_only_a_subfolder_with_documents_is_not_new() {
    let f = fixture();

    let result = RuleSuggestions::over(f.ai_suggestion.clone(), &folder_rule("home"), &f.catalog);

    assert_eq!(result.target_folder, Some(SuggestedFolder { path: path("home"), is_new: false }));
}

#[test]
fn a_document_already_in_a_subfolder_of_the_rule_folder_stays_where_it_is() {
    let f = fixture();
    let in_subfolder = ClassificationSuggestion { document_id: f.filed.id.clone(), ..f.ai_suggestion.clone() };

    let result = RuleSuggestions::over(in_subfolder, &folder_rule("home"), &f.catalog);

    assert_eq!(result.target_folder, None);
    assert_eq!(result.target_reason, None);
}

#[test]
fn a_rule_whose_folder_is_the_top_level_keeps_every_document_in_place_whatever_the_ai_suggests() {
    let f = fixture();
    let rules = RuleResult { target_folder: Some(RelativePath::root()), ..RuleResult::default() };

    let result = RuleSuggestions::over(f.ai_suggestion.clone(), &rules, &f.catalog);

    assert_eq!(result.target_folder, None);
    assert_eq!(result.target_reason, None);
}

#[test]
fn a_rule_note_names_the_rule_or_its_id_when_the_rule_is_gone() {
    let f = fixture();
    let gone = MatchingRuleId::new("rule-gone").expect("valid id");
    let rules = RuleResult {
        category_id: Some(f.bills.id.clone()),
        target_folder: Some(path("home/energy")),
        sources: IndexMap::from([(RuleTarget::Category, f.bill_rule.id.clone()), (RuleTarget::TargetFolder, gone)]),
        ..RuleResult::default()
    };

    let result = RuleSuggestions::over(f.ai_suggestion.clone(), &rules, &f.catalog);

    assert_eq!(
        result.rule_notes,
        IndexMap::from([
            (RuleTarget::Category, "Energy bills".to_owned()),
            (RuleTarget::TargetFolder, "rule-gone".to_owned()),
        ])
    );
}
