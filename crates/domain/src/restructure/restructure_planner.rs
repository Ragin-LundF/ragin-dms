use indexmap::{IndexMap, IndexSet};

use crate::catalog::Catalog;
use crate::document::{Document, DocumentId};
use crate::library::{FileNamePolicy, RelativePath};
use crate::restructure::{
    FileNaming, PathTemplate, PlaceholderName, PlannedMove, RestructurePlan, SkipReason, SkippedDocument,
    TemplateRendering,
};
use crate::settings::InboxSettings;
use crate::{dates, text};

/// Plans a restructure: where each document of a scope belongs by a [`PathTemplate`]. Pure; nothing moves until
/// the user applied the plan.
///
/// A target is never a path another document has now, even one that moves away in the same plan, so the moves
/// can be applied one by one in any order. A clash gets a ` (2)` suffix; names that differ only by case or
/// Unicode normalization clash, as they do on macOS and Windows. A new folder that differs from an existing one
/// only by case takes the existing folder's spelling.
pub struct RestructurePlanner;

impl RestructurePlanner {
    /// # Panics
    /// Never: targets are built from valid folder paths and existing file names.
    #[must_use]
    pub fn plan(
        catalog: &Catalog,
        scope: &[&Document],
        template: &PathTemplate,
        naming: FileNaming,
        inbox: &InboxSettings,
    ) -> RestructurePlan {
        let mut occupied: IndexMap<String, IndexSet<DocumentId>> = IndexMap::new();
        for document in catalog.documents().values() {
            occupied.entry(key(&document.path)).or_default().insert(document.id.clone());
        }
        let folders = existing_folders(catalog);
        let mut plan = RestructurePlan {
            template: template.text().to_owned(),
            moves: Vec::new(),
            unchanged: Vec::new(),
            skipped: Vec::new(),
            renamed_for_clash: IndexSet::new(),
        };
        let mut ordered = scope.to_vec();
        ordered.sort_by(|first, second| text::compare(first.path.value(), second.path.value()));
        for document in ordered {
            if inbox.contains(&document.path) {
                plan.skipped.push(skipped(document, SkipReason::InInbox));
                continue;
            }
            let folder = match template.render(document, catalog) {
                TemplateRendering::Missing(placeholder) => {
                    plan.skipped.push(skipped(document, reason_for(placeholder)));
                    continue;
                }
                TemplateRendering::Rendered(folder) => spelled_like_existing(&folder, &folders),
            };
            let wanted = folder.resolve(&file_name_for(document, naming)).expect("invariant: a valid file name");
            if key(&wanted) == key(&document.path) {
                plan.unchanged.push(document.id.clone());
                continue;
            }
            let target = free_target(&wanted, &occupied);
            if target != wanted {
                plan.renamed_for_clash.insert(document.id.clone());
            }
            occupied.insert(key(&target), IndexSet::from([document.id.clone()]));
            plan.moves.push(PlannedMove { document_id: document.id.clone(), from: document.path.clone(), to: target });
        }
        plan
    }
}

fn skipped(document: &Document, reason: SkipReason) -> SkippedDocument {
    SkippedDocument { document_id: document.id.clone(), path: document.path.clone(), reason }
}

fn free_target(wanted: &RelativePath, occupied: &IndexMap<String, IndexSet<DocumentId>>) -> RelativePath {
    if !occupied.contains_key(&key(wanted)) {
        return wanted.clone();
    }
    let parent = wanted.parent().unwrap_or_default();
    let parent_key = key(&parent);
    let siblings: Vec<&str> = occupied
        .keys()
        .filter(|path| path.rsplit_once('/').map_or("", |(folder, _)| folder) == parent_key)
        .map(|path| path.rsplit('/').next().unwrap_or_default())
        .collect();
    parent
        .resolve(&FileNamePolicy::next_free_name(wanted.file_name(), &siblings))
        .expect("invariant: a free variant of a valid name")
}

fn file_name_for(document: &Document, naming: FileNaming) -> String {
    let (Some(title), Some(date), FileNaming::DateAndTitle) =
        (&document.metadata.title, document.metadata.document_date, naming)
    else {
        return document.path.file_name().to_owned();
    };
    let extension = text::after_last(document.path.file_name(), '.');
    let sanitized = FileNamePolicy::sanitize(&format!("{} {title}", dates::iso(date)));
    // Kotlin's `take` throws for a negative count (an extension of 200+ characters); this keeps nothing instead.
    let room = FileNamePolicy::MAX_LENGTH.saturating_sub(text::utf16_len(extension) + 1);
    let base = text::take_utf16(&sanitized, room);
    if extension.is_empty() { base.to_owned() } else { format!("{base}.{extension}") }
}

/// Every folder the catalog's documents are in, by their comparison key.
fn existing_folders(catalog: &Catalog) -> IndexMap<String, RelativePath> {
    catalog
        .documents()
        .values()
        .flat_map(|document| std::iter::successors(document.path.parent(), RelativePath::parent))
        .filter(|folder| !folder.is_root())
        .map(|folder| (key(&folder), folder))
        .collect()
}

/// `folder` with each level spelled like an existing folder that differs only by case.
fn spelled_like_existing(folder: &RelativePath, existing: &IndexMap<String, RelativePath>) -> RelativePath {
    let mut result = RelativePath::root();
    for segment in folder.segments() {
        let candidate = result.resolve(segment).expect("invariant: a segment of a valid path");
        result = existing.get(&key(&candidate)).cloned().unwrap_or(candidate);
    }
    result
}

fn reason_for(placeholder: PlaceholderName) -> SkipReason {
    match placeholder {
        PlaceholderName::Category | PlaceholderName::CategoryTop => SkipReason::MissingCategory,
        PlaceholderName::Year | PlaceholderName::Month => SkipReason::MissingDate,
        PlaceholderName::Correspondent => SkipReason::MissingCorrespondent,
        PlaceholderName::Field | PlaceholderName::Kind => SkipReason::MissingField,
    }
}

fn key(path: &RelativePath) -> String {
    FileNamePolicy::comparison_key(path.value())
}
