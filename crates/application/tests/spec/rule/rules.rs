//! `RulesTest`.

use dms_application::catalog::CatalogEdit;
use dms_application::classification::{SuggestOutcome, SuggestionApproval};
use dms_application::common::OperationOutcome;
use dms_application::port::ai::{ClassificationResult, DocumentContent};
use dms_application::port::system::Clock;
use dms_domain::classification::{ClassificationSuggestion, SuggestedCategory};
use dms_domain::document::{ClassificationRecord, ClassificationSource, Document, DocumentMetadata};
use dms_domain::rule::{
    Correspondent, MatchingRule, MatchingRuleId, RuleActions, RuleCondition, RuleOrigin, RuleTarget,
};
use dms_domain::task::OpenTask;
use dms_domain::taxonomy::{Category, TagName};
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::test_data::{category_id, tag};

fn bills() -> Category {
    Category::new(category_id("bills"), "Bills", None, None).expect("valid category")
}

fn energy() -> TagName {
    tag("Energy")
}

fn rule(id: &str, name: &str, condition: RuleCondition, actions: RuleActions) -> MatchingRule {
    MatchingRule::new(MatchingRuleId::new(id).expect("valid id"), name, vec![condition], actions, RuleOrigin::Manual)
        .expect("valid rule")
}

fn library_with_rule() -> TestLibrary {
    let library = TestLibrary::new();
    library.open();
    library.store.commit(|catalog| catalog.add_category(bills()));
    let energy_bills = rule(
        "rule-1",
        "Sample Energy bills",
        RuleCondition::CorrespondentIs { name: "Sample Energy".to_owned() },
        RuleActions { category_id: Some(bills().id), tags: IndexSet::from([energy()]), ..RuleActions::default() },
    );
    library.change_settings().run("rule", |settings| {
        settings
            .with_rule(energy_bills.clone())
            .with_correspondent(Correspondent::new("Sample Energy", Vec::new()).expect("valid correspondent"))
    });
    library
}

fn suggested(library: &TestLibrary, document: &Document) -> ClassificationSuggestion {
    match library.suggest_classification().run(&document.id) {
        SuggestOutcome::Suggested(suggestion) => suggestion,
        other => panic!("expected Suggested, got {other:?}"),
    }
}

fn learned_rule_tasks(library: &TestLibrary) -> Vec<OpenTask> {
    let dashboard = library.dashboard(None).expect("a dashboard");
    dashboard.tasks.into_iter().filter(|task| matches!(task, OpenTask::ReviewLearnedRule(_))).collect()
}

fn classified(library: &TestLibrary, document: &Document) {
    let edit = library.edit_catalog();
    edit.run(&CatalogEdit::UpdateMetadata {
        document_id: document.id.clone(),
        metadata: DocumentMetadata { correspondent: Some("Sample Energy".to_owned()), ..DocumentMetadata::default() },
    });
    edit.run(&CatalogEdit::AssignCategory { document_id: document.id.clone(), category_id: Some(bills().id) });
    let record = ClassificationRecord::new(ClassificationSource::Manual, None, library.clock.now()).expect("valid");
    library.store.commit(|catalog| catalog.record_classification(&document.id, record.clone()));
}

fn library_with_three_classified_bills() -> TestLibrary {
    let library = TestLibrary::new();
    library.open();
    library.store.commit(|catalog| catalog.add_category(bills()));
    for index in 0..3 {
        let document = library.add(&format!("bill-{index}.pdf"), &index.to_string());
        classified(&library, &document);
    }
    library
}

#[test]
fn when_rules_decide_category_and_correspondent_the_ai_is_not_asked() {
    let library = library_with_rule();
    let document = library.add("scan.pdf", "a");
    library.reader.content.set(DocumentContent::Text("Sample Energy - your bill".to_owned()));

    let suggestion = suggested(&library, &document);

    assert_eq!(library.classifier.requests.lock().len(), 0);
    assert_eq!(suggestion.model, ClassificationSuggestion::RULES_MODEL);
    assert_eq!(suggestion.category, Some(SuggestedCategory::Existing(bills().id)));
    assert_eq!(suggestion.correspondent.as_deref(), Some("Sample Energy"));
    assert_eq!(suggestion.tags.iter().map(|tag| tag.name.clone()).collect::<Vec<_>>(), vec![energy()]);
    assert_eq!(suggestion.rule_notes.get(&RuleTarget::Category).map(String::as_str), Some("Sample Energy bills"));
}

#[test]
fn an_approved_rule_suggestion_is_recorded_as_made_by_the_rules() {
    let library = library_with_rule();
    let document = library.add("scan.pdf", "a");
    library.reader.content.set(DocumentContent::Text("Sample Energy".to_owned()));
    library.suggest_classification().run(&document.id);

    library
        .approve_suggestion()
        .run(&document.id, &SuggestionApproval { tags: IndexSet::from([energy()]), ..SuggestionApproval::default() });

    let classification = library.catalog().documents()[&document.id].classification.clone();
    assert_eq!(classification.map(|record| record.source), Some(ClassificationSource::Rule));
}

#[test]
fn when_rules_decide_only_part_the_ai_gets_them_as_fixed_and_they_override_its_answer() {
    let library = library_with_rule();
    let other = Category::new(category_id("other"), "Other", None, None).expect("valid category");
    library.store.commit(|catalog| catalog.add_category(other.clone()));
    let document = library.add("scan.pdf", "a");
    library.reader.content.set(DocumentContent::Text("no known sender".to_owned()));
    let manual = rule(
        "rule-2",
        "Scans",
        RuleCondition::FileNameMatches { glob: "scan*".to_owned() },
        RuleActions { category_id: Some(bills().id), ..RuleActions::default() },
    );
    library.change_settings().run("rule", |settings| settings.with_rule(manual.clone()));
    library.classifier.result.set(Some(ClassificationResult::Suggested(ClassificationSuggestion {
        category: Some(SuggestedCategory::Existing(other.id.clone())),
        ..ClassificationSuggestion::empty(document.id.clone(), "test-model")
    })));

    let suggestion = suggested(&library, &document);

    let requests = library.classifier.requests.get();
    assert_eq!(requests.len(), 1);
    let fixed = requests[0].fixed_values.clone().expect("fixed values");
    assert_eq!(fixed.category_id, Some(bills().id));
    assert_eq!(suggestion.category, Some(SuggestedCategory::Existing(bills().id)));
    assert_eq!(suggestion.model, "test-model");
}

#[test]
fn a_learned_proposal_becomes_a_task_and_accepting_it_adds_a_rule_and_the_correspondent() {
    let library = library_with_three_classified_bills();

    let tasks = learned_rule_tasks(&library);
    assert_eq!(tasks.len(), 1);
    let OpenTask::ReviewLearnedRule(proposal) = &tasks[0] else { panic!("expected a learned rule") };
    let outcome = library.accept_rule_proposal().run(proposal);
    assert!(matches!(outcome, OperationOutcome::Done { .. }), "got {outcome:?}");

    let settings = library.catalog().settings().clone();
    assert_eq!(settings.rules.len(), 1);
    assert_eq!(settings.rules[0].origin, RuleOrigin::Learned);
    assert_eq!(settings.rules[0].actions.category_id, Some(bills().id));
    let correspondents: Vec<&str> = settings.correspondents.iter().map(|it| it.name.as_str()).collect();
    assert_eq!(correspondents, vec!["Sample Energy"]);
    assert_eq!(learned_rule_tasks(&library), Vec::new());
}

#[test]
fn a_rejected_proposal_is_not_proposed_again() {
    let library = library_with_three_classified_bills();
    let tasks = learned_rule_tasks(&library);
    assert_eq!(tasks.len(), 1);

    library.dismiss_task().run(&tasks[0].key()).expect("dismissed");

    assert_eq!(learned_rule_tasks(&library), Vec::new());
}
