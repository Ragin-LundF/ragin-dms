use std::collections::HashMap;

use chrono_tz::Tz;
use dms_domain::catalog::Catalog;
use dms_domain::document::Document;
use dms_domain::field::FieldValues;
use dms_domain::library::{FileNamePolicy, RelativePath};
use dms_domain::taxonomy::CategoryId;
use dms_domain::{iso_date, text};

use crate::export::ExportOptions;
use crate::port::export::{ExportBundle, ExportEntry, IndexRow};

/// Lays out an export: the path of every file in the bundle and the index. Pure. Names stay unique the way
/// case-insensitive file systems compare them; a clash gets a ` (2)` suffix.
pub struct ExportPlanner;

impl ExportPlanner {
    /// # Panics
    /// Never for valid documents: their names are valid path segments.
    #[must_use]
    pub fn plan(catalog: &Catalog, documents: &[Document], options: ExportOptions, zone: Tz) -> ExportBundle {
        let mut sorted: Vec<&Document> = documents.iter().collect();
        sorted.sort_by(|first, second| text::compare(first.path.value(), second.path.value()));
        let mut taken: HashMap<String, Vec<String>> = HashMap::new();
        let placed: Vec<(&Document, String)> = sorted
            .into_iter()
            .map(|document| {
                let folder = if options.flatten {
                    RelativePath::root()
                } else {
                    document.path.parent().unwrap_or_else(RelativePath::root)
                };
                let siblings = taken.entry(text::lowercase(folder.value())).or_default();
                let name = FileNamePolicy::next_free_name(&name_of(document, options), siblings);
                siblings.push(name.clone());
                let target = folder.resolve(&name).expect("invariant: a valid file name").value().to_owned();
                (document, target)
            })
            .collect();
        let definitions = &catalog.settings().field_definitions;
        ExportBundle {
            entries: placed
                .iter()
                .map(|(document, target)| ExportEntry { source: document.path.clone(), target: target.clone() })
                .collect(),
            index: options
                .index
                .then(|| placed.iter().map(|(document, target)| row_of(document, target, catalog, zone)).collect()),
            field_labels: definitions.iter().map(|definition| definition.label.clone()).collect(),
        }
    }
}

fn name_of(document: &Document, options: ExportOptions) -> String {
    let file_name = document.path.file_name();
    let Some(title) = document.metadata.title.as_deref().filter(|_| options.rename_to_title) else {
        return file_name.to_owned();
    };
    let extension = text::after_last(file_name, '.');
    // Kotlin's take(negative) threw for an extension of 200+ characters; here the base name is just left out.
    let room = FileNamePolicy::MAX_LENGTH.saturating_sub(text::utf16_len(extension) + 1);
    let sanitized = FileNamePolicy::sanitize(title);
    let base = text::take_utf16(&sanitized, room);
    if extension.is_empty() { base.to_owned() } else { format!("{base}.{extension}") }
}

fn row_of(document: &Document, target: &str, catalog: &Catalog, zone: Tz) -> IndexRow {
    let category = document.category_id.as_ref().and_then(|id| catalog.categories().get(id));
    let mut tags: Vec<_> = document.tags.iter().collect();
    tags.sort();
    let date = document.metadata.document_date.unwrap_or_else(|| document.added_at.with_timezone(&zone).date_naive());
    IndexRow {
        path: target.to_owned(),
        title: document.metadata.title.clone().unwrap_or_default(),
        date: iso_date(date),
        correspondent: document.metadata.correspondent.clone().unwrap_or_default(),
        category: category.map(|category| category_path(&category.id, catalog)).unwrap_or_default(),
        tags: tags.iter().map(|tag| tag.value()).collect::<Vec<_>>().join(", "),
        fields: catalog
            .settings()
            .field_definitions
            .iter()
            .map(|definition| document.fields.get(&definition.key).map(FieldValues::format).unwrap_or_default())
            .collect(),
        note: document.note.clone().unwrap_or_default(),
    }
}

fn category_path(id: &CategoryId, catalog: &Catalog) -> String {
    let categories = catalog.categories();
    let mut names: Vec<&str> = std::iter::successors(categories.get(id), |current| {
        current.parent_id.as_ref().and_then(|it| categories.get(it))
    })
    .take(categories.len())
    .map(|category| category.name.as_str())
    .collect();
    names.reverse();
    names.join(" / ")
}
