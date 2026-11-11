//! What the command palette lists for a query (Kotlin `PaletteItems`): documents by name and by text, and places
//! to go (tags, categories, folders, smart folders) with their counts. The actions are listed by the window, which
//! knows the busy lanes. Only reads.

use std::collections::HashMap;

use dms_application::search::SearchOutcome;
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentId};
use dms_domain::library::RelativePath;
use dms_domain::taxonomy::Category;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::AppState;

const MAX_DOCUMENTS: usize = 6;
const MAX_TEXT_MATCHES: usize = 4;
const MAX_PLACES: usize = 8;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PaletteMatches {
    pub documents: Vec<PaletteDocument>,
    pub places: Vec<PalettePlace>,
}

/// Opens the document in the library; `detail` says where it was found.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PaletteDocument {
    pub document_id: String,
    pub label: String,
    pub detail: String,
}

/// Shows the documents of a place in the library. `kind` is "Tag", "Category", "Folder" or "Smart folder";
/// `value` is the tag, category id, folder path or smart folder id; `count` is `None` for smart folders.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PalettePlace {
    pub kind: String,
    pub value: String,
    pub label: String,
    pub count: Option<u32>,
}

#[tauri::command]
#[specta::specta]
pub async fn palette_matches(ctx: AppState<'_>, query: String) -> Result<PaletteMatches, crate::dto::IpcError> {
    let ctx = ctx.inner().clone();
    crate::session::reporting::blocking(move || {
        let Some(catalog) = ctx.catalog() else { return Ok(PaletteMatches::default()) };
        let text_matches = match ctx.services().map(|services| services.library.search_contents.run(query.trim())) {
            Some(SearchOutcome::Found(ids)) => ids,
            _ => IndexSet::new(),
        };
        Ok(present_palette(&catalog, &query, &text_matches))
    })
    .await
}

pub fn present_palette(catalog: &Catalog, query: &str, text_matches: &IndexSet<DocumentId>) -> PaletteMatches {
    let wanted = query.trim().to_lowercase();
    if wanted.is_empty() {
        return PaletteMatches::default();
    }
    let places =
        places_of(catalog).into_iter().filter(|place| place.label.to_lowercase().contains(&wanted)).take(MAX_PLACES);
    PaletteMatches { documents: documents(catalog, &wanted, text_matches), places: places.collect() }
}

fn documents(catalog: &Catalog, wanted: &str, text_matches: &IndexSet<DocumentId>) -> Vec<PaletteDocument> {
    let by_path =
        |documents: &mut Vec<&Document>| documents.sort_by_key(|document| document.path.value().to_lowercase());
    let mut by_name: Vec<&Document> = catalog
        .documents()
        .values()
        .filter(|document| names_of(document).iter().any(|name| name.contains(wanted)))
        .collect();
    by_path(&mut by_name);
    by_name.truncate(MAX_DOCUMENTS);
    let mut by_text: Vec<&Document> = text_matches
        .iter()
        .filter_map(|id| catalog.documents().get(id))
        .filter(|document| !by_name.iter().any(|named| named.id == document.id))
        .collect();
    by_path(&mut by_text);
    by_text.truncate(MAX_TEXT_MATCHES);
    let item = |document: &Document, detail: String| PaletteDocument {
        document_id: document.id.value().to_owned(),
        label: document.metadata.title.clone().unwrap_or_else(|| document.path.file_name().to_owned()),
        detail,
    };
    by_name
        .iter()
        .map(|document| item(document, document.path.value().to_owned()))
        .chain(by_text.iter().map(|document| item(document, format!("Found in its text · {}", document.path.value()))))
        .collect()
}

/// Every place of the catalog with its count, counted in one pass over the documents.
fn places_of(catalog: &Catalog) -> Vec<PalettePlace> {
    let mut tag_counts: HashMap<&str, u32> = HashMap::new();
    let mut category_counts: HashMap<&str, u32> = HashMap::new();
    let mut folder_counts: HashMap<String, u32> = HashMap::new();
    for document in catalog.documents().values() {
        for tag in &document.tags {
            *tag_counts.entry(tag.value()).or_default() += 1;
        }
        if let Some(category) = &document.category_id {
            *category_counts.entry(category.value()).or_default() += 1;
        }
        for folder in folders_above(&document.path) {
            *folder_counts.entry(folder).or_default() += 1;
        }
    }
    let mut tags: Vec<_> = catalog.tags().iter().collect();
    tags.sort();
    let tags = tags.into_iter().map(|tag| {
        place("Tag", tag.value(), tag.value().to_owned(), Some(tag_counts.get(tag.value()).copied().unwrap_or(0)))
    });
    let mut categories: Vec<PalettePlace> = catalog
        .categories()
        .values()
        .map(|category| {
            let count = category_counts.get(category.id.value()).copied().unwrap_or(0);
            place("Category", category.id.value(), category_path(category, catalog), Some(count))
        })
        .collect();
    categories.sort_by_key(|place| place.label.to_lowercase());
    let mut folders: Vec<(String, u32)> = folder_counts.into_iter().collect();
    folders.sort_by_key(|(folder, _)| folder.to_lowercase());
    let folders = folders.into_iter().map(|(folder, count)| place("Folder", &folder, folder.clone(), Some(count)));
    let smart = catalog
        .settings()
        .saved_searches
        .iter()
        .map(|search| place("Smart folder", search.id.value(), search.name.clone(), None));
    tags.chain(categories).chain(folders).chain(smart).collect()
}

fn place(kind: &str, value: &str, label: String, count: Option<u32>) -> PalettePlace {
    PalettePlace { kind: kind.to_owned(), value: value.to_owned(), label, count }
}

/// "Parent / Child", as Kotlin's `ReviewDrafts.pathOf`.
fn category_path(category: &Category, catalog: &Catalog) -> String {
    let mut names = vec![category.name.as_str()];
    let mut current = category;
    while let Some(parent) = current.parent_id.as_ref().and_then(|id| catalog.categories().get(id)) {
        if names.len() > catalog.categories().len() {
            break;
        }
        names.push(parent.name.as_str());
        current = parent;
    }
    names.reverse();
    names.join(" / ")
}

fn names_of(document: &Document) -> Vec<String> {
    [Some(document.path.file_name()), document.metadata.title.as_deref(), document.metadata.correspondent.as_deref()]
        .into_iter()
        .flatten()
        .map(str::to_lowercase)
        .collect()
}

/// Every folder above a file, without the root.
fn folders_above(path: &RelativePath) -> Vec<String> {
    let mut folders = Vec::new();
    let mut current = path.parent();
    while let Some(folder) = current {
        if folder.is_root() {
            break;
        }
        folders.push(folder.value().to_owned());
        current = folder.parent();
    }
    folders
}

#[cfg(test)]
mod tests {
    use dms_domain::catalog::{Catalog, CatalogUpdate, LibraryId};
    use dms_domain::document::{ContentHash, Document, DocumentId};
    use dms_domain::library::RelativePath;
    use dms_domain::taxonomy::{Category, CategoryId, TagName};
    use indexmap::IndexSet;

    use super::present_palette;

    fn applied(update: CatalogUpdate) -> Catalog {
        match update {
            CatalogUpdate::Applied(catalog) => catalog,
            CatalogUpdate::Rejected(reason) => panic!("expected an applied update, got {reason:?}"),
        }
    }

    fn catalog() -> Catalog {
        let mut catalog = Catalog::empty(LibraryId::new("library-1").expect("id"));
        let finance = Category::new(CategoryId::new("c-1").expect("id"), "Finance", None, None).expect("category");
        let tax =
            Category::new(CategoryId::new("c-2").expect("id"), "Tax", Some(CategoryId::new("c-1").expect("id")), None)
                .expect("category");
        catalog = applied(catalog.add_category(finance).expect("add"));
        catalog = applied(catalog.add_category(tax).expect("add"));
        for (index, path) in ["Taxes/2025/Return.pdf", "Taxes/Notes.txt", "Car invoice.pdf"].iter().enumerate() {
            let mut document = Document::new(
                DocumentId::new(format!("d-{index}")).expect("id"),
                RelativePath::of(path).expect("path"),
                ContentHash::new(format!("{index:064x}")).expect("hash"),
                100,
                "2024-01-01T10:00:00Z".parse().expect("instant"),
            )
            .expect("document");
            if index == 0 {
                document.category_id = Some(CategoryId::new("c-2").expect("id"));
                document.tags.insert(TagName::of("taxes").expect("tag"));
            }
            catalog = applied(catalog.add_document(document).expect("add"));
        }
        catalog
    }

    #[test]
    fn lists_documents_by_name_then_by_text_and_places_with_counts() {
        let text = IndexSet::from([DocumentId::new("d-2").expect("id"), DocumentId::new("d-0").expect("id")]);
        let by_name = present_palette(&catalog(), " NOTES", &text);
        let names: Vec<_> =
            by_name.documents.iter().map(|document| (document.label.as_str(), document.detail.as_str())).collect();
        assert_eq!(
            names,
            vec![
                ("Notes.txt", "Taxes/Notes.txt"),
                ("Car invoice.pdf", "Found in its text · Car invoice.pdf"),
                ("Return.pdf", "Found in its text · Taxes/2025/Return.pdf"),
            ]
        );
        // Folder names are no document names: "tax" finds the documents only through their text.
        let matches = present_palette(&catalog(), "tax", &IndexSet::new());
        assert_eq!(matches.documents, Vec::new());
        let places: Vec<_> =
            matches.places.iter().map(|place| (place.kind.as_str(), place.label.as_str(), place.count)).collect();
        assert_eq!(
            places,
            vec![
                ("Tag", "taxes", Some(1)),
                ("Category", "Finance / Tax", Some(1)),
                ("Folder", "Taxes", Some(2)),
                ("Folder", "Taxes/2025", Some(1))
            ]
        );
    }

    #[test]
    fn an_empty_query_lists_nothing() {
        assert_eq!(present_palette(&catalog(), "  ", &IndexSet::new()), super::PaletteMatches::default());
    }
}
