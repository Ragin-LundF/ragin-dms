use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentMetadata};
use dms_domain::field::{FieldPresets, FieldValue};
use dms_domain::restructure::{PathTemplate, PlaceholderName, TemplateParse, TemplateProblem, TemplateRendering};
use dms_domain::settings::CatalogSettings;
use dms_domain::taxonomy::Category;
use indexmap::indexmap;

use crate::test_data::{self, Applied, category_id, date, path};

fn insurance() -> Category {
    Category::new(category_id("insurance"), "Insurance", None, None).expect("valid")
}

fn car() -> Category {
    Category::new(category_id("car"), "Car: Policies", Some(insurance().id), None).expect("valid")
}

fn document() -> Document {
    Document { category_id: Some(car().id), ..test_data::document("policy", "inbox/policy.pdf") }
}

fn catalog() -> Catalog {
    test_data::catalog_with(&[])
        .add_category(insurance())
        .applied()
        .add_category(car())
        .applied()
        .change_settings(CatalogSettings { field_definitions: FieldPresets::all(), ..CatalogSettings::default() })
        .applied()
        .add_document(document())
        .applied()
        .update_metadata(
            &document().id,
            DocumentMetadata {
                title: None,
                document_date: Some(date(2024, 3, 5)),
                correspondent: Some("Sample/Insurer".to_owned()),
            },
        )
        .applied()
}

fn render(template: &str, catalog: &Catalog) -> TemplateRendering {
    let TemplateParse::Parsed(parsed) = PathTemplate::parse(template) else {
        panic!("invalid template: {template}");
    };
    parsed.render(&catalog.documents()[&document().id], catalog)
}

fn folder(template: &str) -> String {
    match render(template, &catalog()) {
        TemplateRendering::Rendered(folder) => folder.value().to_owned(),
        TemplateRendering::Missing(placeholder) => panic!("missing {placeholder:?}"),
    }
}

#[test]
fn the_category_becomes_one_folder_per_level_with_invalid_characters_replaced() {
    assert_eq!(folder("{category}/{year}/"), "Insurance/Car- Policies/2024");
    assert_eq!(folder("{category.top}/{year}-{month}"), "Insurance/2024-03");
}

#[test]
fn values_never_add_folders_of_their_own() {
    assert_eq!(folder("{correspondent}"), "Sample-Insurer");
    assert_eq!(folder(" /{kind}/Archive/ "), "Pdf/Archive");
}

#[test]
fn a_missing_value_uses_the_fallback_otherwise_it_is_reported() {
    let undated = catalog().update_metadata(&document().id, DocumentMetadata::default()).applied();

    assert_eq!(render("{year|Undated}", &undated), TemplateRendering::Rendered(path("Undated")));
    assert_eq!(render("{category}/{year}", &undated), TemplateRendering::Missing(PlaceholderName::Year));
}

#[test]
fn field_values_can_be_used() {
    let with_field = catalog()
        .set_fields(&document().id, indexmap! { FieldPresets::due_date().key => FieldValue::Date(date(2026, 1, 31)) })
        .applied();

    assert_eq!(render("Due/{field:due}", &with_field), TemplateRendering::Rendered(path("Due/2026-01-31")));
}

#[test]
fn broken_templates_are_refused_with_a_reason() {
    let problems = [
        ("", TemplateProblem::Empty),
        ("{category", TemplateProblem::UnclosedBrace),
        ("{colour}", TemplateProblem::UnknownPlaceholder),
        ("{field:Not A Key}", TemplateProblem::InvalidFieldKey),
        ("../{year}", TemplateProblem::InvalidFolderName),
        ("a:b/{year}", TemplateProblem::InvalidFolderName),
    ];

    for (raw, problem) in problems {
        assert_eq!(PathTemplate::parse(raw), TemplateParse::Invalid(problem), "{raw}");
    }
}

#[test]
fn an_overlong_field_key_is_an_invalid_key_where_kotlin_threw() {
    let raw = format!("{{field:{}}}", "k".repeat(41));

    assert_eq!(PathTemplate::parse(&raw), TemplateParse::Invalid(TemplateProblem::InvalidFieldKey));
}
