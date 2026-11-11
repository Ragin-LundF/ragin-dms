use dms_domain::catalog::Catalog;
use dms_domain::document::{ClassificationRecord, ClassificationSource, Document, DocumentMetadata};
use dms_domain::rule::{
    MatchingRule, MatchingRuleId, RuleActions, RuleCondition, RuleLearner, RuleOrigin, RuleProposal,
};
use dms_domain::taxonomy::{Category, TagName};
use indexmap::{IndexSet, indexset};

use crate::test_data::{self, Applied, category_id, instant, tag};

fn bills() -> Category {
    Category::new(category_id("bills"), "Bills", None, None).expect("valid")
}

fn other() -> Category {
    Category::new(category_id("other"), "Other", None, None).expect("valid")
}

fn energy() -> TagName {
    tag("Energy")
}

fn catalog_of(categories: &[Category], tags: &[IndexSet<TagName>], classified: bool) -> Catalog {
    let base = test_data::catalog_with(&[]).add_category(bills()).applied().add_category(other()).applied();
    categories.iter().enumerate().fold(base, |catalog, (index, category)| {
        let document = Document {
            category_id: Some(category.id.clone()),
            tags: tags.get(index).cloned().unwrap_or_default(),
            ..test_data::document(&format!("d{index}"), &format!("bills/{index}.pdf"))
        };
        let added = catalog
            .add_document(document.clone())
            .applied()
            .update_metadata(
                &document.id,
                DocumentMetadata { correspondent: Some("Sample Energy".to_owned()), ..DocumentMetadata::default() },
            )
            .applied();
        if classified {
            let record = ClassificationRecord::new(ClassificationSource::Manual, None, instant("1970-01-01T00:00:00Z"))
                .expect("valid record");
            added.record_classification(&document.id, record).applied()
        } else {
            added
        }
    })
}

#[test]
fn three_approved_documents_with_the_same_category_make_a_proposal() {
    let proposals = RuleLearner::proposals(&catalog_of(&[bills(), bills(), bills()], &[], true));

    assert_eq!(
        proposals,
        vec![RuleProposal::new("Sample Energy", Some(bills().id), IndexSet::new(), 3).expect("valid proposal")]
    );
}

#[test]
fn two_documents_unapproved_ones_or_a_split_category_make_none() {
    assert_eq!(
        RuleLearner::proposals(&catalog_of(&[bills(), bills()], &[], true)),
        [] as [dms_domain::rule::RuleProposal; 0]
    );
    assert_eq!(
        RuleLearner::proposals(&catalog_of(&[bills(), bills(), bills()], &[], false)),
        [] as [dms_domain::rule::RuleProposal; 0]
    );
    assert_eq!(
        RuleLearner::proposals(&catalog_of(&[bills(), bills(), other()], &[], true)),
        [] as [dms_domain::rule::RuleProposal; 0]
    );
}

#[test]
fn tags_shared_by_at_least_80_percent_are_proposed_too() {
    let tags = [
        indexset! { energy() },
        indexset! { energy() },
        indexset! { energy() },
        indexset! { energy() },
        IndexSet::new(),
    ];

    let proposals = RuleLearner::proposals(&catalog_of(&[bills(), bills(), bills(), bills(), other()], &tags, true));

    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].tags, indexset! { energy() });
    assert_eq!(proposals[0].category_id, None);
}

#[test]
fn a_correspondent_that_already_has_a_rule_is_not_proposed_again() {
    let catalog = catalog_of(&[bills(), bills(), bills()], &[], true);
    let rule = MatchingRule::new(
        MatchingRuleId::new("r").expect("valid"),
        "Sample Energy",
        vec![RuleCondition::CorrespondentIs { name: "sample energy".to_owned() }],
        RuleActions { category_id: Some(bills().id), ..RuleActions::default() },
        RuleOrigin::Manual,
    )
    .expect("valid rule");

    let with_rule = catalog.change_settings(catalog.settings().with_rule(rule)).applied();

    assert_eq!(RuleLearner::proposals(&with_rule), [] as [dms_domain::rule::RuleProposal; 0]);
}
