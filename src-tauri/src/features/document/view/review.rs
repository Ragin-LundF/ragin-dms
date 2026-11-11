//! The review of an AI suggestion (Kotlin `ReviewDrafts`, `ReviewFields`, `ReviewController.open`): the rows the
//! dialog shows, and the approval the checked rows make.

use dms_application::classification::{ApprovedCategory, SuggestionApproval};
use dms_domain::catalog::Catalog;
use dms_domain::classification::{ClassificationSuggestion, SuggestedCategory};
use dms_domain::document::Document;
use dms_domain::field::{FieldKey, FieldValues};
use dms_domain::library::{FileNamePolicy, RelativePath};
use dms_domain::rule::RuleTarget;
use dms_domain::taxonomy::{Category, TagName};
use dms_domain::{iso_date, text};

use super::texts::{DATE_PROBLEM, parse_date};
use crate::features::document::dto::{Review, ReviewRow, ReviewRowDraft};
use crate::messages;

const SEPARATOR: &str = " / ";
const FIELD_PREFIX: &str = "field:";

pub fn present_review(
    catalog: &Catalog,
    document: &Document,
    suggestion: &ClassificationSuggestion,
    accept_file_names: bool,
) -> Review {
    let confidence = suggestion
        .confidence
        .map(|confidence| format!(" · {} % sure", (confidence * 100.0).round()))
        .unwrap_or_default();
    let source = if suggestion.by_rules() { "your matching rules, without the AI" } else { suggestion.model.as_str() };
    Review {
        document_id: document.id.value().to_owned(),
        document_name: document.path.value().to_owned(),
        source: format!("From {source}{confidence}. Only the checked values are applied."),
        rows: rows_of(catalog, document, suggestion, accept_file_names),
    }
}

fn rows_of(
    catalog: &Catalog,
    document: &Document,
    suggestion: &ClassificationSuggestion,
    accept_file_names: bool,
) -> Vec<ReviewRow> {
    let metadata = &document.metadata;
    let rule_hint = |target: RuleTarget| suggestion.rule_notes.get(&target).map(|rule| format!("From rule: {rule}"));
    let mut rows = Vec::new();
    if let Some(title) = &suggestion.title {
        rows.push(row("title", "Title", title, metadata.title.clone().unwrap_or_default()));
    }
    if let Some(date) = suggestion.document_date {
        rows.push(row(
            "documentDate",
            "Date (YYYY-MM-DD)",
            &iso_date(date),
            metadata.document_date.map(iso_date).unwrap_or_default(),
        ));
    }
    if let Some(correspondent) = &suggestion.correspondent {
        rows.push(row(
            "correspondent",
            "Correspondent",
            correspondent,
            metadata.correspondent.clone().unwrap_or_default(),
        ));
    }
    let current_category = document.category_id.as_ref().and_then(|id| catalog.categories().get(id));
    rows.push(ReviewRow {
        options: category_paths(catalog),
        hint: rule_hint(RuleTarget::Category),
        ..row(
            "category",
            "Category: pick one or type a new name",
            &suggested_category_text(suggestion.category.as_ref(), catalog).unwrap_or_default(),
            current_category.map(|category| category_path(category, catalog)).unwrap_or_default(),
        )
    });
    if !suggestion.tags.is_empty() {
        let tags: Vec<&str> = suggestion.tags.iter().map(|tag| tag.name.value()).collect();
        let mut current: Vec<&TagName> = document.tags.iter().collect();
        current.sort();
        let current: Vec<&str> = current.into_iter().map(TagName::value).collect();
        rows.push(ReviewRow {
            hint: rule_hint(RuleTarget::Tags),
            ..row("tags", "Tags to add, comma separated", &tags.join(", "), current.join(", "))
        });
    }
    if let Some(file_name) = &suggestion.file_name {
        let mut file_row = row("fileName", "File name", file_name, document.path.file_name().to_owned());
        file_row.accepted &= accept_file_names;
        rows.push(file_row);
    }
    if let Some(target) = &suggestion.target_folder {
        let reason = suggestion
            .target_reason
            .as_ref()
            .map(|reason| format!("Why: {reason}"))
            .or_else(|| rule_hint(RuleTarget::TargetFolder));
        let hints: Vec<String> = target.is_new.then(|| "a new folder".to_owned()).into_iter().chain(reason).collect();
        rows.push(ReviewRow {
            options: folder_paths(catalog),
            hint: Some(hints.join(" · ")).filter(|hint| !hint.is_empty()),
            ..row("targetFolder", "Move to folder", target.path.value(), current_folder(document))
        });
    }
    for (key, value) in &suggestion.fields {
        let label = catalog.settings().field(key).map_or(key.value(), |definition| definition.label.as_str());
        let current = document.fields.get(key).map(FieldValues::format).unwrap_or_default();
        rows.push(ReviewRow {
            hint: rule_hint(RuleTarget::Fields),
            ..row(&format!("{FIELD_PREFIX}{}", key.value()), label, &FieldValues::format(value), current)
        });
    }
    rows
}

/// A suggested value; one the AI left empty starts unchecked.
fn row(key: &str, label: &str, value: &str, current: String) -> ReviewRow {
    ReviewRow {
        key: key.to_owned(),
        label: label.to_owned(),
        value: value.to_owned(),
        accepted: !text::is_blank(value),
        current,
        options: Vec::new(),
        hint: None,
    }
}

fn current_folder(document: &Document) -> String {
    document.path.parent().map(|parent| parent.value().to_owned()).unwrap_or_default()
}

/// The approval for the checked rows, or why it cannot be made (the first problem, in row order).
pub fn approval_of(
    catalog: &Catalog,
    document: &Document,
    rows: &[ReviewRowDraft],
) -> Result<SuggestionApproval, String> {
    let taken = |key: &str| {
        rows.iter()
            .find(|row| row.key == key && row.accepted)
            .map(|row| text::trim(&row.value).to_owned())
            .filter(|value| !value.is_empty())
    };
    let document_date = taken("documentDate").map(|raw| parse_date(&raw).ok_or(DATE_PROBLEM)).transpose()?;
    let category = taken("category").map(|raw| approved_category(&raw, catalog)).transpose()?;
    let tags = taken("tags").map(|raw| tags_of(&raw)).transpose()?.unwrap_or_default();
    let file_name = taken("fileName").map(|name| checked_file_name(&name)).transpose()?;
    let target_folder = taken("targetFolder").map(|raw| folder_of(&raw)).transpose()?;
    let mut approval = SuggestionApproval {
        title: taken("title"),
        document_date,
        correspondent: taken("correspondent"),
        category,
        tags: tags.into_iter().collect(),
        file_name: file_name.filter(|name| name != document.path.file_name()),
        target_folder: target_folder.filter(|folder| folder.value() != current_folder(document)),
        ..SuggestionApproval::default()
    };
    let currency = catalog.settings().layout.default_currency.as_ref();
    for row in rows {
        let Some(key) = row.key.strip_prefix(FIELD_PREFIX) else {
            continue;
        };
        let Some(raw) = taken(&row.key) else {
            continue;
        };
        let Some(definition) = FieldKey::new(key).ok().and_then(|key| catalog.settings().field(&key)) else {
            continue;
        };
        let value = FieldValues::parse(definition.field_type, &raw, currency)
            .ok_or_else(|| format!("\"{}\" does not fit the field's type", definition.label))?;
        approval.fields.insert(definition.key.clone(), value);
    }
    Ok(approval)
}

/// An existing category by path or name, ignoring case; otherwise the typed path, whose missing levels approving
/// creates.
fn approved_category(typed: &str, catalog: &Catalog) -> Result<ApprovedCategory, String> {
    let wanted = text::lowercase(typed);
    let existing = catalog.categories().values().find(|category| {
        text::lowercase(&category_path(category, catalog)) == wanted || text::lowercase(&category.name) == wanted
    });
    if let Some(category) = existing {
        return Ok(ApprovedCategory::Existing(category.id.clone()));
    }
    let names: Vec<String> = typed.split('/').map(|name| text::trim(name).to_owned()).collect();
    if names.iter().any(String::is_empty) {
        return Err(format!("Every level of the category \"{typed}\" needs a name"));
    }
    Ok(ApprovedCategory::Path(names))
}

fn tags_of(typed: &str) -> Result<Vec<TagName>, String> {
    typed
        .split(',')
        .filter(|raw| !text::is_blank(raw))
        .map(|raw| {
            TagName::parse(raw)
                .ok_or_else(|| format!("Tag \"{}\" is longer than {} characters", text::trim(raw), TagName::MAX_LENGTH))
        })
        .collect()
}

fn checked_file_name(name: &str) -> Result<String, String> {
    match FileNamePolicy::check(name) {
        Some(violation) => Err(messages::name_problem(violation).to_owned()),
        None => Ok(name.to_owned()),
    }
}

fn folder_of(typed: &str) -> Result<RelativePath, String> {
    let segments: Vec<&str> = typed.split('/').map(text::trim).filter(|segment| !segment.is_empty()).collect();
    for name in &segments {
        if let Some(violation) = FileNamePolicy::check(name) {
            return Err(format!("Folder \"{name}\": {}", messages::name_problem(violation)));
        }
    }
    RelativePath::from_segments(&segments).map_err(|error| error.to_string())
}

/// Every category as a path like `Parent / Child`, sorted ignoring case.
pub fn category_paths(catalog: &Catalog) -> Vec<String> {
    let mut paths: Vec<String> =
        catalog.categories().values().map(|category| category_path(category, catalog)).collect();
    sort_ignoring_case(&mut paths);
    paths
}

/// Every folder the catalog's documents are in, sorted ignoring case.
pub fn folder_paths(catalog: &Catalog) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    for document in catalog.documents().values() {
        let mut folder = document.path.parent();
        while let Some(current) = folder.filter(|folder| !folder.is_root()) {
            if !paths.iter().any(|path| path == current.value()) {
                paths.push(current.value().to_owned());
            }
            folder = current.parent();
        }
    }
    sort_ignoring_case(&mut paths);
    paths
}

fn sort_ignoring_case(values: &mut [String]) {
    values.sort_by(|first, second| text::compare(&text::lowercase(first), &text::lowercase(second)));
}

/// `Parent / Child` for `category`.
pub fn category_path(category: &Category, catalog: &Catalog) -> String {
    let mut names = vec![category.name.as_str()];
    let mut current = category;
    while let Some(parent) = current.parent_id.as_ref().and_then(|id| catalog.categories().get(id)) {
        if names.len() >= catalog.categories().len() {
            break;
        }
        names.push(parent.name.as_str());
        current = parent;
    }
    names.reverse();
    names.join(SEPARATOR)
}

fn suggested_category_text(category: Option<&SuggestedCategory>, catalog: &Catalog) -> Option<String> {
    match category? {
        SuggestedCategory::Existing(id) => {
            catalog.categories().get(id).map(|category| category_path(category, catalog))
        }
        SuggestedCategory::New { name, parent_id } => {
            let parent = parent_id.as_ref().and_then(|id| catalog.categories().get(id));
            Some(match parent {
                Some(parent) => format!("{}{SEPARATOR}{name}", category_path(parent, catalog)),
                None => name.clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use dms_domain::classification::{SuggestedFolder, SuggestedTag};
    use dms_domain::field::FieldType;
    use dms_domain::settings::CatalogSettings;

    use super::*;
    use crate::features::document::view::test_catalog::*;

    fn library() -> (Catalog, Document) {
        let mut letter = with_tags(document(1, "Inbox/scan.pdf"), &["b-tag", "A-tag"]);
        letter.metadata.title = Some("Scan".to_owned());
        let other = document(2, "Home/Power/old.pdf");
        let categories =
            vec![category("c1", "Finance", None), category("c2", "Tax", Some("c1")), category("c3", "home", None)];
        (catalog(vec![letter.clone(), other], categories, vec![], standard_settings()), letter)
    }

    fn drafts(review: &Review) -> Vec<ReviewRowDraft> {
        review
            .rows
            .iter()
            .map(|row| ReviewRowDraft { key: row.key.clone(), accepted: row.accepted, value: row.value.clone() })
            .collect()
    }

    fn set(rows: &mut [ReviewRowDraft], key: &str, value: &str) {
        let row = rows.iter_mut().find(|row| row.key == key).expect("row");
        row.value = value.to_owned();
        row.accepted = true;
    }

    #[test]
    fn rows_show_only_suggested_values_but_always_the_category() {
        let (catalog, letter) = library();
        let mut suggestion = suggestion("d1");
        suggestion.title = Some("Tax assessment".to_owned());
        suggestion.confidence = Some(0.876);
        let review = present_review(&catalog, &letter, &suggestion, false);

        assert_eq!(review.source, "From test-model · 88 % sure. Only the checked values are applied.");
        let keys: Vec<&str> = review.rows.iter().map(|row| row.key.as_str()).collect();
        assert_eq!(keys, ["title", "category"]);
        let category = &review.rows[1];
        assert_eq!((category.value.as_str(), category.accepted), ("", false));
        assert_eq!(category.options, ["Finance", "Finance / Tax", "home"]);
        assert_eq!(review.rows[0].current, "Scan");
    }

    #[test]
    fn rows_carry_current_values_rule_hints_and_folder_reasons() {
        let (catalog, letter) = library();
        let mut suggestion = suggestion("d1");
        suggestion.model = ClassificationSuggestion::RULES_MODEL.to_owned();
        suggestion.category =
            Some(SuggestedCategory::New { name: "Assessments".to_owned(), parent_id: Some(category_id("c2")) });
        suggestion.tags = vec![SuggestedTag { name: tag("tax"), is_new: true }];
        suggestion.file_name = Some("2026 Tax.pdf".to_owned());
        suggestion.target_folder =
            Some(SuggestedFolder::new(RelativePath::of("Finance/Tax").expect("path"), true).expect("folder"));
        suggestion.rule_notes.insert(RuleTarget::Category, "Tax office".to_owned());
        suggestion.rule_notes.insert(RuleTarget::TargetFolder, "Tax office".to_owned());
        suggestion.fields.insert(
            FieldKey::new("amount").expect("key"),
            FieldValues::parse(FieldType::Money, "80", catalog.settings().layout.default_currency.as_ref())
                .expect("money"),
        );

        let review = present_review(&catalog, &letter, &suggestion, false);
        assert!(review.source.starts_with("From your matching rules, without the AI."));
        let row = |key: &str| review.rows.iter().find(|row| row.key == key).expect("row").clone();
        assert_eq!(row("category").value, "Finance / Tax / Assessments");
        assert_eq!(row("category").hint.as_deref(), Some("From rule: Tax office"));
        assert_eq!(row("tags").current, "A-tag, b-tag");
        assert!(!row("fileName").accepted, "file names start unchecked without the setting");
        let folder = row("targetFolder");
        assert_eq!(
            (folder.current.as_str(), folder.hint.as_deref()),
            ("Inbox", Some("a new folder · From rule: Tax office"))
        );
        assert_eq!(folder.options, ["Home", "Home/Power", "Inbox"]);
        let amount = row("field:amount");
        assert_eq!((amount.label.as_str(), amount.value.as_str(), amount.current.as_str()), ("Amount", "80 EUR", ""));

        suggestion.target_reason = Some("Tax papers live there".to_owned());
        let with_reason = present_review(&catalog, &letter, &suggestion, true);
        let folder = with_reason.rows.iter().find(|row| row.key == "targetFolder").expect("row");
        assert_eq!(folder.hint.as_deref(), Some("a new folder · Why: Tax papers live there"));
        assert!(with_reason.rows.iter().any(|row| row.key == "fileName" && row.accepted));
    }

    #[test]
    fn approval_takes_checked_trimmed_values_and_drops_unchanged_name_and_folder() {
        let (catalog, letter) = library();
        let mut suggestion = suggestion("d1");
        suggestion.title = Some("Tax".to_owned());
        suggestion.correspondent = Some("Tax office".to_owned());
        suggestion.tags = vec![SuggestedTag { name: tag("tax"), is_new: false }];
        suggestion.file_name = Some("scan.pdf".to_owned());
        suggestion.target_folder =
            Some(SuggestedFolder::new(RelativePath::of("Inbox").expect("path"), false).expect("folder"));
        let review = present_review(&catalog, &letter, &suggestion, true);
        let mut rows = drafts(&review);
        set(&mut rows, "title", "  Tax 2025 ");
        rows.iter_mut().find(|row| row.key == "correspondent").expect("row").accepted = false;
        set(&mut rows, "category", "finance / tax");
        set(&mut rows, "tags", "tax, , Tax, 2025");

        let approval = approval_of(&catalog, &letter, &rows).expect("approval");
        assert_eq!(approval.title.as_deref(), Some("Tax 2025"));
        assert_eq!(approval.correspondent, None);
        assert_eq!(approval.category, Some(ApprovedCategory::Existing(category_id("c2"))));
        assert_eq!(approval.tags.iter().map(TagName::value).collect::<Vec<_>>(), ["tax", "2025"]);
        assert_eq!(approval.file_name, None, "the current name is not applied");
        assert_eq!(approval.target_folder, None, "the current folder is not applied");
    }

    #[test]
    fn approval_finds_a_category_by_its_name_or_creates_the_typed_path() {
        let (catalog, letter) = library();
        let rows = |category: &str| {
            vec![ReviewRowDraft { key: "category".to_owned(), accepted: true, value: category.to_owned() }]
        };
        assert_eq!(
            approval_of(&catalog, &letter, &rows("TAX")).expect("ok").category,
            Some(ApprovedCategory::Existing(category_id("c2")))
        );
        assert_eq!(
            approval_of(&catalog, &letter, &rows("Home / Garden")).expect("ok").category,
            Some(ApprovedCategory::Path(vec!["Home".to_owned(), "Garden".to_owned()]))
        );
        assert_eq!(
            approval_of(&catalog, &letter, &rows("Home / ")).map(|approval| approval.category),
            Err("Every level of the category \"Home /\" needs a name".to_owned())
        );
    }

    #[test]
    fn approval_names_the_first_problem() {
        let (catalog, letter) = library();
        let refused = |key: &str, value: &str| {
            approval_of(
                &catalog,
                &letter,
                &[ReviewRowDraft { key: key.to_owned(), accepted: true, value: value.to_owned() }],
            )
            .map(|_| ())
        };
        assert_eq!(refused("documentDate", "1.2.2026"), Err(DATE_PROBLEM.to_owned()));
        let long = "x".repeat(61);
        assert_eq!(
            refused("tags", &format!("ok, {long}")),
            Err(format!("Tag \"{long}\" is longer than 60 characters"))
        );
        assert_eq!(refused("fileName", "a|b.pdf"), Err("The name contains one of < > : \" / \\ | ? *".to_owned()));
        assert_eq!(refused("targetFolder", "Home/ nul "), Err("Folder \"nul\": Windows reserves this name".to_owned()));
        assert_eq!(refused("field:amount", "lots"), Err("\"Amount\" does not fit the field's type".to_owned()));
        assert_eq!(refused("field:gone", "lots"), Ok(()), "a field the library no longer has is left out");
        let unchecked =
            ReviewRowDraft { key: "documentDate".to_owned(), accepted: false, value: "nonsense".to_owned() };
        assert!(approval_of(&catalog, &letter, &[unchecked]).is_ok(), "unchecked values are not read");
    }

    #[test]
    fn approval_moves_to_a_typed_folder_with_clean_segments() {
        let (catalog, letter) = library();
        let rows =
            [ReviewRowDraft { key: "targetFolder".to_owned(), accepted: true, value: "/Home/ Power /".to_owned() }];
        assert_eq!(
            approval_of(&catalog, &letter, &rows).expect("ok").target_folder,
            Some(RelativePath::of("Home/Power").expect("path"))
        );
    }

    #[test]
    fn category_paths_are_sorted_ignoring_case() {
        let categories =
            vec![category("c1", "beta", None), category("c2", "Alpha", None), category("c3", "child", Some("c1"))];
        let catalog = catalog(vec![], categories, vec![], CatalogSettings::default());
        assert_eq!(category_paths(&catalog), ["Alpha", "beta", "beta / child"]);
    }
}
