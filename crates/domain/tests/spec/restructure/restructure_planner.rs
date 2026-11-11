use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentMetadata};
use dms_domain::restructure::{
    FileNaming, PathTemplate, PlannedMove, RestructurePlan, RestructurePlanner, SkipReason, TemplateParse,
};
use dms_domain::settings::InboxSettings;
use dms_domain::taxonomy::Category;
use indexmap::indexset;

use crate::test_data::{self, Applied, category_id, date, path};

fn tax() -> Category {
    Category::new(category_id("tax"), "Tax", None, None).expect("valid")
}

fn template() -> PathTemplate {
    match PathTemplate::parse("{category}/{year}") {
        TemplateParse::Parsed(template) => template,
        TemplateParse::Invalid(problem) => panic!("invalid template: {problem:?}"),
    }
}

fn taxed(id: &str, raw: &str) -> Document {
    Document { category_id: Some(tax().id), ..test_data::document(id, raw) }
}

fn catalog_of(documents: &[Document]) -> Catalog {
    let base = test_data::catalog_with(&[]).add_category(tax()).applied();
    documents.iter().fold(base, |catalog, document| {
        catalog
            .add_document(document.clone())
            .applied()
            .update_metadata(
                &document.id,
                DocumentMetadata { document_date: Some(date(2024, 5, 1)), ..DocumentMetadata::default() },
            )
            .applied()
    })
}

fn plan(catalog: &Catalog, naming: FileNaming, inbox: &InboxSettings) -> RestructurePlan {
    let scope: Vec<&Document> = catalog.documents().values().collect();
    RestructurePlanner::plan(catalog, &scope, &template(), naming, inbox)
}

fn plan_keep(catalog: &Catalog) -> RestructurePlan {
    plan(catalog, FileNaming::Keep, &InboxSettings::default())
}

fn targets(plan: &RestructurePlan) -> Vec<&str> {
    plan.moves.iter().map(|planned| planned.to.value()).collect()
}

#[test]
fn documents_move_to_their_template_folder_and_those_in_place_stay() {
    let loose = taxed("loose", "loose.pdf");
    let placed = taxed("placed", "Tax/2024/placed.pdf");

    let plan = plan_keep(&catalog_of(&[loose.clone(), placed.clone()]));

    assert_eq!(
        plan.moves,
        vec![PlannedMove { document_id: loose.id, from: loose.path, to: path("Tax/2024/loose.pdf") }]
    );
    assert_eq!(plan.unchanged, vec![placed.id]);
}

#[test]
fn documents_without_the_values_the_template_needs_are_skipped_with_a_reason() {
    let plan = plan_keep(&catalog_of(&[test_data::document("uncategorized", "a.pdf")]));

    assert_eq!(plan.skipped.iter().map(|it| it.reason).collect::<Vec<_>>(), vec![SkipReason::MissingCategory]);
}

#[test]
fn a_name_clash_gets_a_numbered_suffix_also_against_a_document_that_only_differs_by_case() {
    let first = taxed("first", "one/scan.pdf");
    let second = taxed("second", "two/SCAN.pdf");

    let plan = plan_keep(&catalog_of(&[first, second.clone()]));

    assert_eq!(targets(&plan), vec!["Tax/2024/scan.pdf", "Tax/2024/SCAN (2).pdf"]);
    assert_eq!(plan.renamed_for_clash, indexset! { second.id });
}

#[test]
fn names_that_differ_only_by_unicode_normalization_clash() {
    let nfc = taxed("nfc", "Tax/2024/Gr\u{fc}\u{df}e.pdf");
    let nfd = taxed("nfd", "old/Gru\u{308}\u{df}e.pdf");

    let plan = plan_keep(&catalog_of(&[nfc, nfd]));

    assert_eq!(targets(&plan), vec!["Tax/2024/Gr\u{fc}\u{df}e (2).pdf"]);
}

#[test]
fn a_target_folder_that_differs_only_by_case_takes_the_existing_spelling() {
    let existing = taxed("existing", "TAX/2024/kept.pdf");
    let loose = taxed("loose", "loose.pdf");

    let plan = plan_keep(&catalog_of(&[existing.clone(), loose]));

    assert_eq!(targets(&plan), vec!["TAX/2024/loose.pdf"]);
    assert_eq!(plan.unchanged, vec![existing.id]);
}

#[test]
fn the_naming_convention_renames_to_date_and_title() {
    let loose = taxed("loose", "scan_0001.PDF");
    let catalog = catalog_of(std::slice::from_ref(&loose))
        .update_metadata(
            &loose.id,
            DocumentMetadata {
                title: Some("Tax return".to_owned()),
                document_date: Some(date(2024, 5, 1)),
                correspondent: None,
            },
        )
        .applied();

    let plan = plan(&catalog, FileNaming::DateAndTitle, &InboxSettings::default());

    assert_eq!(targets(&plan), vec!["Tax/2024/2024-05-01 Tax return.PDF"]);
}

#[test]
fn inbox_documents_are_never_restructured() {
    let inboxed = taxed("inboxed", "Inbox/new.pdf");

    let plan =
        plan(&catalog_of(&[inboxed]), FileNaming::Keep, &InboxSettings { enabled: true, ..InboxSettings::default() });

    assert_eq!(plan.skipped.iter().map(|it| it.reason).collect::<Vec<_>>(), vec![SkipReason::InInbox]);
}

#[test]
fn unticking_moves_keeps_the_rest_of_the_plan() {
    let first = taxed("first", "a/x.pdf");
    let second = taxed("second", "b/x.pdf");

    let plan = plan_keep(&catalog_of(&[first, second.clone()]))
        .only(|planned| planned.from.parent().map(|parent| parent.value().to_owned()) != Some("a".to_owned()));

    assert_eq!(plan.moves.iter().map(|planned| planned.document_id.clone()).collect::<Vec<_>>(), vec![second.id]);
    assert_eq!(plan.source_folders(), indexset! { path("b") });
}
