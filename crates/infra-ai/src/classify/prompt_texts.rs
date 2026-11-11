//! The words sent to the model for a classification. The system message holds the rules and the library's taxonomy,
//! the user message the document. The model gets the existing names, so it prefers what the library uses.

use dms_application::port::ai::ClassificationRequest;
use dms_domain::field::FieldType;
use dms_domain::iso_date;
use dms_domain::library::RelativePath;
use dms_domain::taxonomy::{Category, TagName};
use dms_domain::text;

use super::naming_convention;
use crate::prompt::{self, tax_tag_of};

const MAX_LISTED: usize = 200;

pub(crate) fn system(request: &ClassificationRequest) -> String {
    let extension = request.current_file_name.rsplit_once('.').map_or("", |(_, extension)| extension);
    let mut categories: Vec<String> =
        request.categories.iter().map(|category| path_of(category, &request.categories)).collect();
    categories.sort_by(|first, second| text::compare(first, second));
    let mut tags: Vec<&str> = request.tags.iter().map(TagName::value).collect();
    tags.sort_by(|first, second| text::compare(first, second));
    prompt::CLASSIFICATION.fill(&[
        ("extension", extension),
        ("categories", &listed(&categories)),
        ("tags", &listed(&tags)),
        ("taxTag", &tax_tag_of(request.tags.iter().map(TagName::value))),
        ("fields", &fields_of(request)),
        ("convention", &naming_convention::describe(&request.file_name_examples)),
        ("examples", &listed(&request.file_name_examples)),
        ("today", &iso_date(request.today)),
    ])
}

/// The user message; `text` `None` means the document goes as page images.
pub(crate) fn user(request: &ClassificationRequest, text: Option<&str>) -> String {
    let folders: Vec<&str> = request.folders.iter().map(RelativePath::value).collect();
    prompt::CLASSIFICATION_REQUEST.fill(&[
        ("fileName", &request.current_file_name),
        ("folders", &listed(&folders)),
        ("outline", &outline_section(request)),
        ("rules", &rules_section(request)),
        ("document", &prompt::document_section(text)),
    ])
}

/// The folder outline for a target-folder suggestion, one folder per line with its document count, main category
/// and sample titles; or a note that no target folder is wanted.
pub(crate) fn outline_section(request: &ClassificationRequest) -> String {
    let Some(outline) = &request.outline else {
        return prompt::CLASSIFICATION_OUTLINE_NONE.text();
    };
    if outline.folders.is_empty() {
        return prompt::CLASSIFICATION_OUTLINE_EMPTY.text();
    }
    let lines: Vec<String> = outline
        .folders
        .iter()
        .map(|folder| {
            let category = folder
                .top_category
                .as_ref()
                .and_then(|id| request.categories.iter().find(|category| &category.id == id))
                .map(|category| path_of(category, &request.categories));
            let mut parts = vec![folder.path.value().to_owned(), format!("{} documents", folder.document_count)];
            parts.extend(category.map(|category| format!("mostly {category}")));
            if !folder.sample_titles.is_empty() {
                parts.push(format!("e.g. {}", folder.sample_titles.join(", ")));
            }
            if folder.hidden_subfolders > 0 {
                parts.push(format!("{} more subfolders", folder.hidden_subfolders));
            }
            parts.join(" | ")
        })
        .collect();
    prompt::CLASSIFICATION_OUTLINE.fill(&[("folders", &lines.join("\n"))])
}

/// What the user's matching rules decided; the model is asked to keep it, and it overrides the answer.
pub(crate) fn rules_section(request: &ClassificationRequest) -> String {
    let Some(rules) = &request.fixed_values else {
        return String::new();
    };
    let category = rules
        .category_id
        .as_ref()
        .and_then(|id| request.categories.iter().find(|category| &category.id == id))
        .map(|category| path_of(category, &request.categories));
    let mut parts = Vec::new();
    parts.extend(category.map(|category| format!("category: {category}")));
    parts.extend(rules.correspondent.as_ref().map(|correspondent| format!("correspondent: {correspondent}")));
    if !rules.tags.is_empty() {
        let tags: Vec<&str> = rules.tags.iter().map(TagName::value).collect();
        parts.push(format!("tags: {}", tags.join(", ")));
    }
    if parts.is_empty() {
        return String::new();
    }
    prompt::CLASSIFICATION_RULES.fill(&[("values", &parts.join("; "))])
}

/// The custom fields as `key (Label, TYPE)`, with the default currency for money fields.
fn fields_of(request: &ClassificationRequest) -> String {
    let currency = request
        .default_currency
        .as_ref()
        .map(|currency| format!(" in {currency} unless stated otherwise"))
        .unwrap_or_default();
    let fields: Vec<String> = request
        .field_definitions
        .iter()
        .map(|definition| {
            let field_type = match definition.field_type {
                FieldType::Money => format!("MONEY{currency}"),
                other => type_name(other).to_owned(),
            };
            format!("{} ({}, {field_type})", definition.key.value(), definition.label)
        })
        .collect();
    listed(&fields)
}

fn type_name(field_type: FieldType) -> &'static str {
    match field_type {
        FieldType::Text => "TEXT",
        FieldType::Number => "NUMBER",
        FieldType::Money => "MONEY",
        FieldType::Date => "DATE",
        FieldType::Boolean => "BOOLEAN",
    }
}

/// "Parent / Child" for nested categories, the way the user sees them.
pub(crate) fn path_of(category: &Category, categories: &[Category]) -> String {
    let mut names = vec![category.name.as_str()];
    let mut current = category;
    // At most as many steps as there are categories, so a cycle cannot loop forever.
    while names.len() < categories.len() {
        let Some(parent) =
            current.parent_id.as_ref().and_then(|parent_id| categories.iter().find(|other| &other.id == parent_id))
        else {
            break;
        };
        names.push(parent.name.as_str());
        current = parent;
    }
    names.reverse();
    names.join(" / ")
}

fn listed<S: AsRef<str>>(values: &[S]) -> String {
    if values.is_empty() {
        return "none yet".to_owned();
    }
    values.iter().take(MAX_LISTED).map(AsRef::as_ref).collect::<Vec<_>>().join("; ")
}
