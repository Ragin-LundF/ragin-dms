//! The quick look of one document (Kotlin `QuickFacts`, `SuggestionValues`, the category menu entries of
//! `LibraryTrees` and `QuickLookSheet`'s inputs).

use chrono::NaiveDate;
use dms_domain::catalog::Catalog;
use dms_domain::classification::{ClassificationSuggestion, SuggestedCategory};
use dms_domain::document::{Document, DocumentKind};
use dms_domain::field::{FieldDefinition, FieldRole, FieldType, FieldValue, FieldValues};
use dms_domain::taxonomy::{Category, CategoryId};
use dms_domain::{iso_date, text};

use super::details::present_details;
use super::texts::relative_day;
use crate::features::document::dto::{CategoryChoice, FieldFact, QuickFacts, QuickLook};

/// What the quick look needs besides the catalog and the document.
pub struct QuickLookContext<'a> {
    pub suggestion: Option<&'a ClassificationSuggestion>,
    pub primary_folder_id: Option<String>,
    /// The device setting "Rename to the suggested file name"; only then the file name is a chip.
    pub accept_file_names: bool,
    pub today: NaiveDate,
}

pub fn present_quick_look(catalog: &Catalog, document: &Document, context: &QuickLookContext<'_>) -> QuickLook {
    let kind = document.kind();
    let folder = document.path.parent().map(|parent| parent.value().to_owned()).filter(|value| !value.is_empty());
    QuickLook {
        document_id: document.id.value().to_owned(),
        path: document.path.value().to_owned(),
        file_name: document.path.file_name().to_owned(),
        title: document.metadata.title.clone(),
        kind: kind_name(kind).to_owned(),
        readable_by_ai: kind.readable_by_ai(),
        primary_folder_id: context.primary_folder_id.clone(),
        facts: facts_of(document, catalog, context.today),
        suggestion: context
            .suggestion
            .map(|suggestion| suggestion_values(suggestion, catalog, context.accept_file_names)),
        categories: category_choices(catalog),
        library_tags: sorted_tags(catalog),
        folder: folder.unwrap_or_else(|| "/".to_owned()),
        details: present_details(catalog, document),
    }
}

pub fn kind_name(kind: DocumentKind) -> &'static str {
    match kind {
        DocumentKind::Pdf => "pdf",
        DocumentKind::Image => "image",
        DocumentKind::Office => "office",
        DocumentKind::Other => "other",
    }
}

/// The key facts: category, tags, correspondent, date, and the amount and due date when the library has such
/// fields.
pub fn facts_of(document: &Document, catalog: &Catalog, today: NaiveDate) -> QuickFacts {
    let mut tags: Vec<_> = document.tags.iter().collect();
    tags.sort();
    let fact = |definition: &FieldDefinition| field_fact(definition, document, today);
    QuickFacts {
        category: document
            .category_id
            .as_ref()
            .and_then(|id| catalog.categories().get(id))
            .map(|category| category.name.clone()),
        tags: tags.into_iter().map(|tag| tag.value().to_owned()).collect(),
        correspondent: document.metadata.correspondent.clone(),
        document_date: document.metadata.document_date.map(iso_date),
        amount: amount_field(catalog).map(fact),
        due: due_field(catalog).map(fact),
    }
}

/// The first money field; the quick look shows it as the amount.
pub fn amount_field(catalog: &Catalog) -> Option<&FieldDefinition> {
    catalog.settings().field_definitions.iter().find(|field| field.field_type == FieldType::Money)
}

/// The first field with the due-date role.
pub fn due_field(catalog: &Catalog) -> Option<&FieldDefinition> {
    catalog.settings().field_definitions.iter().find(|field| field.role == Some(FieldRole::DueDate))
}

fn field_fact(definition: &FieldDefinition, document: &Document, today: NaiveDate) -> FieldFact {
    let label = definition.label.clone();
    let Some(value) = document.fields.get(&definition.key) else {
        return FieldFact { label, value: None, chip: None, urgent: false };
    };
    let text = Some(FieldValues::format(value));
    match (value, definition.role) {
        (FieldValue::Date(date), Some(role)) => FieldFact {
            label,
            value: text,
            chip: Some(relative_day(*date, today)),
            urgent: (*date - today).num_days() <= i64::from(role.default_lead_days()),
        },
        _ => FieldFact { label, value: text, chip: None, urgent: false },
    }
}

/// The values of a suggestion as short chip texts, in the order the review shows them: exactly what accepting it
/// unchanged applies, so the file name only with `accept_file_name`.
pub fn suggestion_values(
    suggestion: &ClassificationSuggestion,
    catalog: &Catalog,
    accept_file_name: bool,
) -> Vec<String> {
    let filled = |value: &Option<String>| value.as_deref().filter(|value| !text::is_blank(value)).map(str::to_owned);
    let mut values = Vec::new();
    values.extend(filled(&suggestion.title).map(|title| format!("title: {title}")));
    values.extend(suggestion.category.as_ref().map(|category| category_text(category, catalog)));
    values.extend(filled(&suggestion.correspondent).map(|name| format!("correspondent: {name}")));
    values.extend(suggestion.document_date.map(|date| format!("date: {}", iso_date(date))));
    if accept_file_name {
        values.extend(filled(&suggestion.file_name).map(|name| format!("name: {name}")));
    }
    values.extend(suggestion.target_folder.as_ref().map(|folder| format!("folder: {}", folder.path.value())));
    values.extend(suggestion.tags.iter().map(|tag| format!("+ {}", tag.name.value())));
    values.extend(suggestion.fields.iter().map(|(key, value)| {
        let label = catalog.settings().field(key).map_or(key.value(), |definition| definition.label.as_str());
        format!("{label}: {}", FieldValues::format(value))
    }));
    values
}

fn category_text(category: &SuggestedCategory, catalog: &Catalog) -> String {
    match category {
        SuggestedCategory::Existing(id) => {
            let name = catalog.categories().get(id).map_or(id.value(), |category| category.name.as_str());
            format!("category: {name}")
        }
        SuggestedCategory::New { name, .. } => format!("new category: {name}"),
    }
}

/// Every category, each level sorted by name ignoring case, subcategories right below their parent.
pub fn category_choices(catalog: &Catalog) -> Vec<CategoryChoice> {
    let mut choices = Vec::new();
    add_children(catalog, None, 0, &mut choices);
    choices
}

fn add_children(catalog: &Catalog, parent: Option<&CategoryId>, depth: u32, choices: &mut Vec<CategoryChoice>) {
    let mut children: Vec<&Category> =
        catalog.categories().values().filter(|category| category.parent_id.as_ref() == parent).collect();
    children.sort_by(|first, second| text::compare(&text::lowercase(&first.name), &text::lowercase(&second.name)));
    for category in children {
        choices.push(CategoryChoice { id: category.id.value().to_owned(), name: category.name.clone(), depth });
        add_children(catalog, Some(&category.id), depth + 1, choices);
    }
}

fn sorted_tags(catalog: &Catalog) -> Vec<String> {
    let mut tags: Vec<_> = catalog.tags().iter().collect();
    tags.sort();
    tags.into_iter().map(|tag| tag.value().to_owned()).collect()
}

#[cfg(test)]
mod tests {
    use dms_domain::classification::{SuggestedFolder, SuggestedTag};
    use dms_domain::field::FieldKey;
    use dms_domain::library::RelativePath;
    use dms_domain::settings::CatalogSettings;

    use super::*;
    use crate::features::document::view::test_catalog::*;

    fn context(today: &str) -> QuickLookContext<'static> {
        QuickLookContext {
            suggestion: None,
            primary_folder_id: Some("f1".to_owned()),
            accept_file_names: false,
            today: day(today),
        }
    }

    #[test]
    fn facts_show_category_name_sorted_tags_and_iso_date() {
        let mut letter = with_tags(document(1, "Home/letter.pdf"), &["zebra", "Apple", "mango"]);
        letter.category_id = Some(category_id("c1"));
        letter.metadata.document_date = Some(day("2026-02-03"));
        letter.metadata.correspondent = Some("City Power".to_owned());
        let catalog =
            catalog(vec![letter.clone()], vec![category("c1", "Home", None)], vec![], CatalogSettings::default());

        let facts = facts_of(&letter, &catalog, day("2026-03-01"));

        assert_eq!(facts.category.as_deref(), Some("Home"));
        assert_eq!(facts.tags, ["Apple", "mango", "zebra"]);
        assert_eq!(facts.correspondent.as_deref(), Some("City Power"));
        assert_eq!(facts.document_date.as_deref(), Some("2026-02-03"));
        assert_eq!(facts.amount, None, "no money field defined");
        assert_eq!(facts.due, None);
    }

    #[test]
    fn due_fact_gets_a_relative_chip_that_is_urgent_within_the_lead_days_and_when_past() {
        let settings = standard_settings();
        let mut soon = document(1, "a.pdf");
        field_value(&mut soon, "due", "2026-03-08", &settings);
        let mut later = document(2, "b.pdf");
        field_value(&mut later, "due", "2026-03-09", &settings);
        let mut past = document(3, "c.pdf");
        field_value(&mut past, "due", "2026-02-20", &settings);
        field_value(&mut past, "amount", "12,50", &settings);
        let catalog = catalog(vec![soon.clone(), later.clone(), past.clone()], vec![], vec![], settings);
        let today = day("2026-03-01");

        let soon_due = facts_of(&soon, &catalog, today).due.expect("due fact");
        assert_eq!((soon_due.chip.as_deref(), soon_due.urgent), (Some("in 7 days"), true));
        let later_due = facts_of(&later, &catalog, today).due.expect("due fact");
        assert_eq!((later_due.chip.as_deref(), later_due.urgent), (Some("in 8 days"), false));
        let past_facts = facts_of(&past, &catalog, today);
        let past_due = past_facts.due.expect("due fact");
        assert_eq!(
            (past_due.label.as_str(), past_due.value.as_deref(), past_due.urgent),
            ("Due date", Some("2026-02-20"), true)
        );
        let amount = past_facts.amount.expect("amount fact");
        assert_eq!((amount.value.as_deref(), amount.chip), (Some("12.50 EUR"), None));
    }

    #[test]
    fn missing_field_value_has_no_value_and_no_chip() {
        let letter = document(1, "a.pdf");
        let catalog = catalog(vec![letter.clone()], vec![], vec![], standard_settings());
        let due = facts_of(&letter, &catalog, day("2026-03-01")).due.expect("due fact");
        assert_eq!(due, FieldFact { label: "Due date".to_owned(), value: None, chip: None, urgent: false });
    }

    #[test]
    fn suggestion_chips_follow_the_review_order_and_leave_out_blank_values() {
        let settings = standard_settings();
        let letter = document(1, "Inbox/scan.pdf");
        let catalog = catalog(vec![letter.clone()], vec![category("c1", "Home", None)], vec![], settings);
        let mut suggestion = suggestion("d1");
        suggestion.title = Some("Power bill".to_owned());
        suggestion.category = Some(SuggestedCategory::Existing(category_id("c1")));
        suggestion.correspondent = Some("  ".to_owned());
        suggestion.document_date = Some(day("2026-02-01"));
        suggestion.file_name = Some("2026-02 Power bill.pdf".to_owned());
        suggestion.target_folder =
            Some(SuggestedFolder::new(RelativePath::of("Home/Power").expect("path"), true).expect("folder"));
        suggestion.tags = vec![SuggestedTag { name: tag("invoice"), is_new: true }];
        suggestion.fields.insert(
            FieldKey::new("amount").expect("key"),
            FieldValues::parse(FieldType::Money, "80 EUR", None).expect("money"),
        );
        suggestion.fields.insert(FieldKey::new("gone").expect("key"), FieldValue::Flag(true));

        let without_name = suggestion_values(&suggestion, &catalog, false);
        assert_eq!(
            without_name,
            [
                "title: Power bill",
                "category: Home",
                "date: 2026-02-01",
                "folder: Home/Power",
                "+ invoice",
                "Amount: 80 EUR",
                "gone: true"
            ]
        );
        let with_name = suggestion_values(&suggestion, &catalog, true);
        assert_eq!(with_name[3], "name: 2026-02 Power bill.pdf");

        suggestion.category = Some(SuggestedCategory::New { name: "Utilities".to_owned(), parent_id: None });
        assert_eq!(suggestion_values(&suggestion, &catalog, false)[1], "new category: Utilities");
    }

    #[test]
    fn category_choices_sort_each_level_ignoring_case_with_children_below_their_parent() {
        let categories = vec![
            category("c1", "home", None),
            category("c2", "Finance", None),
            category("c3", "tax", Some("c2")),
            category("c4", "Bank", Some("c2")),
            category("c5", "Garden", Some("c1")),
        ];
        let catalog = catalog(vec![], categories, vec![], CatalogSettings::default());
        let choices: Vec<(String, u32)> =
            category_choices(&catalog).into_iter().map(|choice| (choice.name, choice.depth)).collect();
        let expected = [("Finance", 0), ("Bank", 1), ("tax", 1), ("home", 0), ("Garden", 1)];
        assert_eq!(choices, expected.map(|(name, depth)| (name.to_owned(), depth)));
    }

    #[test]
    fn quick_look_starts_move_at_the_folder_or_the_top_level() {
        let nested = document(1, "Home/Power/bill.pdf");
        let top = document(2, "notes.txt");
        let catalog = catalog(vec![nested.clone(), top.clone()], vec![], vec![], CatalogSettings::default());
        let nested_look = present_quick_look(&catalog, &nested, &context("2026-03-01"));
        assert_eq!(
            (nested_look.folder.as_str(), nested_look.kind.as_str(), nested_look.readable_by_ai),
            ("Home/Power", "pdf", true)
        );
        let top_look = present_quick_look(&catalog, &top, &context("2026-03-01"));
        assert_eq!((top_look.folder.as_str(), top_look.kind.as_str(), top_look.readable_by_ai), ("/", "other", false));
        assert_eq!(top_look.suggestion, None);
    }
}
