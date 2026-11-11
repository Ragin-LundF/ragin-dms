use dms_domain::document::{Document, DocumentMetadata};
use dms_domain::rule::{
    Correspondent, MatchingRule, MatchingRuleId, RuleActions, RuleCondition, RuleMatcher, RuleOrigin, RuleTarget,
};
use dms_domain::taxonomy::CategoryId;
use indexmap::indexset;

use crate::test_data::{self, category_id, path, tag};

fn energy() -> Correspondent {
    Correspondent::new("Sample Energy", vec!["SE Power".to_owned()]).expect("valid")
}

fn bills() -> CategoryId {
    category_id("bills")
}

fn contracts() -> CategoryId {
    category_id("contracts")
}

fn document() -> Document {
    test_data::document("doc", "inbox/bill-2024-03.pdf")
}

fn rule(
    id: &str,
    conditions: Vec<RuleCondition>,
    actions: RuleActions,
    origin: RuleOrigin,
    enabled: bool,
) -> MatchingRule {
    let rule = MatchingRule::new(MatchingRuleId::new(id).expect("valid"), id, conditions, actions, origin)
        .expect("valid rule");
    MatchingRule { enabled, ..rule }
}

fn learned(id: &str, condition: RuleCondition, actions: RuleActions) -> MatchingRule {
    rule(id, vec![condition], actions, RuleOrigin::Learned, true)
}

fn words(words: &[&str], require_all: bool) -> RuleCondition {
    RuleCondition::TextContains { words: words.iter().map(|word| (*word).to_owned()).collect(), require_all }
}

fn glob(glob: &str) -> RuleCondition {
    RuleCondition::FileNameMatches { glob: glob.to_owned() }
}

fn to_folder(raw: &str) -> RuleActions {
    RuleActions { target_folder: Some(path(raw)), ..RuleActions::default() }
}

fn to_category(id: CategoryId) -> RuleActions {
    RuleActions { category_id: Some(id), ..RuleActions::default() }
}

#[test]
fn a_correspondent_is_recognized_by_name_or_alias_as_a_whole_word() {
    let by_alias = RuleMatcher::apply(&[], &[energy()], &document(), "Invoice from se power ltd");
    let part_of_word = RuleMatcher::apply(&[], &[energy()], &document(), "SE Powerful");

    assert_eq!(by_alias.correspondent.as_deref(), Some("Sample Energy"));
    assert_eq!(part_of_word.correspondent, None);
}

#[test]
fn each_condition_holds_on_its_own_terms() {
    let by_correspondent =
        learned("c", RuleCondition::CorrespondentIs { name: "sample energy".to_owned() }, to_category(bills()));
    let all_words = learned(
        "all",
        words(&["meter", "kWh"], true),
        RuleActions { tags: indexset! { tag("Energy") }, ..RuleActions::default() },
    );
    let any_word = learned(
        "any",
        words(&["gas", "water"], false),
        RuleActions { tags: indexset! { tag("Utility") }, ..RuleActions::default() },
    );
    let by_glob = learned("glob", glob("BILL-????-*.pdf"), to_folder("Home/Bills"));

    let result = RuleMatcher::apply(
        &[by_correspondent, all_words, any_word, by_glob],
        &[energy()],
        &document(),
        "Sample Energy: meter reading 120 kWh",
    );

    assert_eq!(result.category_id, Some(bills()));
    assert_eq!(result.tags, indexset! { tag("Energy") });
    assert_eq!(result.target_folder, Some(path("Home/Bills")));
    assert!(result.is_sufficient());
}

#[test]
fn the_more_specific_rule_wins_a_conflict_then_the_manual_one() {
    let broad = rule("broad", vec![words(&["contract"], false)], to_category(bills()), RuleOrigin::Manual, true);
    let specific = learned("specific", words(&["contract", "insurance"], true), to_category(contracts()));
    let learned_rule = learned("learned", glob("*.pdf"), to_folder("Learned"));
    let manual = rule("manual", vec![glob("bill*")], to_folder("Manual"), RuleOrigin::Manual, true);

    let result = RuleMatcher::apply(&[broad, specific, learned_rule, manual], &[], &document(), "insurance contract");

    assert_eq!(result.category_id, Some(contracts()));
    assert_eq!(result.target_folder, Some(path("Manual")));
    assert_eq!(result.sources.get(&RuleTarget::Category), Some(&MatchingRuleId::new("specific").expect("valid")));
}

#[test]
fn a_disabled_rule_is_not_used_and_the_documents_own_correspondent_counts() {
    let disabled = rule(
        "off",
        vec![RuleCondition::CorrespondentIs { name: "Own Sender".to_owned() }],
        to_category(bills()),
        RuleOrigin::Learned,
        false,
    );
    let named = Document {
        metadata: DocumentMetadata { correspondent: Some("Own Sender".to_owned()), ..DocumentMetadata::default() },
        ..document()
    };

    let result = RuleMatcher::apply(&[disabled], &[], &named, "");

    assert_eq!(result.correspondent.as_deref(), Some("Own Sender"));
    assert_eq!(result.category_id, None);
    assert!(!result.is_sufficient());
}

#[test]
fn a_glob_ignores_only_ascii_case_like_the_java_regex() {
    let umlaut = rule("u", vec![glob("\u{c4}*")], to_folder("U"), RuleOrigin::Manual, true);
    let lower = Document { path: path("\u{e4}rger.pdf"), ..document() };

    assert_eq!(RuleMatcher::apply(&[umlaut], &[], &lower, "").target_folder, None);
}
