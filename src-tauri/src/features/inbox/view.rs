//! The inbox as the screen shows it (Kotlin `InboxViewModel.toUiState`), and the folders filed into last.

use std::sync::Mutex;

use dms_domain::catalog::Catalog;
use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::Document;
use dms_domain::library::RelativePath;
use dms_domain::restructure::{PathTemplate, TemplateParse, TemplateRendering};
use dms_domain::text;

use super::dto::{InboxRowDto, InboxStateDto};

const RECENT_TARGETS: usize = 5;

/// The folders filed into last, newest first; kept for this run of the app like the Kotlin view model's list.
static RECENT: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn recent_targets() -> Vec<String> {
    crate::session::lock(&RECENT).clone()
}

/// Puts `folder` first among the recent targets, at most five.
pub fn remember_target(folder: &str) {
    let mut recent = crate::session::lock(&RECENT);
    *recent = with_recent(&recent, folder);
}

fn with_recent(recent: &[String], folder: &str) -> Vec<String> {
    std::iter::once(folder.to_owned())
        .chain(recent.iter().filter(|target| *target != folder).cloned())
        .take(RECENT_TARGETS)
        .collect()
}

/// The inbox of the open `catalog`; `suggestions` are the AI suggestions waiting for review.
pub fn present_inbox(
    catalog: Option<&Catalog>,
    suggestions: &[ClassificationSuggestion],
    ai_enabled: bool,
    recent: Vec<String>,
) -> InboxStateDto {
    let Some(catalog) = catalog else {
        return InboxStateDto { recent_targets: recent, ..InboxStateDto::default() };
    };
    let layout = &catalog.settings().layout;
    let inbox = &layout.inbox;
    let template = layout.path_template.as_deref().and_then(|raw| match PathTemplate::parse(raw) {
        TemplateParse::Parsed(template) => Some(template),
        TemplateParse::Invalid(_) => None,
    });
    let mut documents: Vec<&Document> =
        catalog.documents().values().filter(|document| inbox.contains(&document.path)).collect();
    documents.sort_by_key(|document| std::cmp::Reverse(document.added_at));
    let rows = documents
        .into_iter()
        .map(|document| {
            let prefill = template.as_ref().and_then(|template| match template.render(document, catalog) {
                TemplateRendering::Rendered(folder) => Some(folder.value().to_owned()),
                TemplateRendering::Missing(_) => None,
            });
            let suggestion = suggestions.iter().find(|suggestion| suggestion.document_id == document.id);
            row(document, &inbox.path, suggestion, prefill)
        })
        .collect();
    let inbox_path = inbox.path.value();
    InboxStateDto {
        enabled: inbox.enabled,
        path: inbox_path.to_owned(),
        ai_analysis: inbox.ai_analysis && ai_enabled,
        rows,
        folders: folder_paths(catalog)
            .into_iter()
            .filter(|folder| folder != inbox_path && !folder.starts_with(&format!("{inbox_path}/")))
            .collect(),
        recent_targets: recent,
    }
}

fn row(
    document: &Document,
    inbox: &RelativePath,
    suggestion: Option<&ClassificationSuggestion>,
    prefill: Option<String>,
) -> InboxRowDto {
    let parent = document.path.parent().unwrap_or_else(RelativePath::root);
    let subfolder = if &parent == inbox {
        String::new()
    } else {
        let value = parent.value();
        value.strip_prefix(&format!("{}/", inbox.value())).unwrap_or(value).to_owned()
    };
    let suggestion_note = match suggestion {
        None => String::new(),
        Some(suggestion) => match &suggestion.target_folder {
            Some(folder) => format!(" · AI suggests {}", folder.path),
            None => " · AI suggestion waiting".to_owned(),
        },
    };
    InboxRowDto {
        document_id: document.id.value().to_owned(),
        name: document.path.file_name().to_owned(),
        subfolder,
        added_at: document.added_at.to_rfc3339(),
        size: document.size,
        has_suggestion: suggestion.is_some(),
        suggestion_note,
        analyzable: document.kind().readable_by_ai(),
        prefill: prefill.filter(|folder| !folder.is_empty()),
        email: dms_application::mail::is_mail_file(document.path.file_name()),
    }
}

/// Every folder that holds a document, directly or below, sorted ignoring case (Kotlin `ReviewDrafts.folderPathsOf`).
fn folder_paths(catalog: &Catalog) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut sorted = Vec::new();
    for document in catalog.documents().values() {
        let mut folder = document.path.parent();
        while let Some(current) = folder.filter(|path| !path.is_root()) {
            folder = current.parent();
            if seen.insert(current.value().to_owned()) {
                sorted.push(current.value().to_owned());
            }
        }
    }
    sorted.sort_by(|first, second| text::compare(&text::lowercase(first), &text::lowercase(second)));
    sorted
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use dms_domain::catalog::{CatalogUpdate, LibraryId};
    use dms_domain::classification::SuggestedFolder;
    use dms_domain::document::{ContentHash, DocumentId};
    use dms_domain::settings::{CatalogSettings, InboxSettings, LibraryLayout};

    use super::*;

    fn applied(update: CatalogUpdate) -> Catalog {
        match update {
            CatalogUpdate::Applied(catalog) => catalog,
            CatalogUpdate::Rejected(reason) => panic!("rejected: {reason:?}"),
        }
    }

    fn document(id: &str, path: &str, seed: u32, added: &str) -> Document {
        let added: DateTime<Utc> = added.parse().expect("instant");
        Document::new(
            DocumentId::new(id).expect("id"),
            RelativePath::of(path).expect("path"),
            ContentHash::new(format!("{seed:064x}")).expect("hash"),
            1_000,
            added,
        )
        .expect("document")
    }

    fn catalog(inbox_enabled: bool, template: Option<&str>) -> Catalog {
        let documents = [
            document("d1", "Inbox/old.pdf", 1, "2026-01-01T10:00:00Z"),
            document("d2", "Inbox/Scans/new.pdf", 2, "2026-03-01T10:00:00Z"),
            document("d3", "Home/Bills/power.pdf", 3, "2026-02-01T10:00:00Z"),
            document("d4", "Inboxes/not-inside.zip", 4, "2026-04-01T10:00:00Z"),
            document("d5", "archive/loose.pdf", 5, "2026-02-02T10:00:00Z"),
        ];
        let catalog = documents
            .into_iter()
            .fold(Catalog::empty(LibraryId::new("library-1").expect("id")), |catalog, document| {
                applied(catalog.add_document(document).expect("valid"))
            });
        let layout = LibraryLayout {
            inbox: InboxSettings { enabled: inbox_enabled, ..InboxSettings::default() },
            path_template: template.map(str::to_owned),
            ..LibraryLayout::default()
        };
        applied(catalog.change_settings(CatalogSettings { layout, ..CatalogSettings::default() }).expect("valid"))
    }

    #[allow(clippy::default_trait_access)] // IndexMap is not a dependency of this crate
    fn suggestion(document: &str, folder: Option<&str>) -> ClassificationSuggestion {
        ClassificationSuggestion {
            document_id: DocumentId::new(document).expect("id"),
            model: "test-model".to_owned(),
            title: None,
            category: None,
            tags: Vec::new(),
            document_date: None,
            correspondent: None,
            file_name: None,
            transcription: None,
            confidence: None,
            target_folder: folder
                .map(|path| SuggestedFolder::new(RelativePath::of(path).expect("path"), false).expect("folder")),
            target_reason: None,
            fields: Default::default(),
            rule_notes: Default::default(),
            tax: None,
            e_invoice: Vec::new(),
            requested_actions: Vec::new(),
        }
    }

    #[test]
    fn rows_are_the_inbox_documents_newest_first_with_their_subfolder() {
        let state = present_inbox(Some(&catalog(true, None)), &[], true, Vec::new());
        let rows: Vec<_> = state.rows.iter().map(|row| (row.name.as_str(), row.subfolder.as_str())).collect();
        assert_eq!(rows, vec![("new.pdf", "Scans"), ("old.pdf", "")]);
        assert_eq!(state.path, "Inbox");
        assert!(state.enabled && state.ai_analysis);
    }

    #[test]
    fn a_switched_off_inbox_has_no_rows_and_ai_analysis_needs_ai() {
        let state = present_inbox(Some(&catalog(false, None)), &[], false, Vec::new());
        assert!(!state.enabled);
        assert_eq!(state.rows, Vec::new());
        assert!(!state.ai_analysis);
        assert_eq!(present_inbox(None, &[], true, vec!["A".to_owned()]).recent_targets, vec!["A".to_owned()]);
    }

    #[test]
    fn waiting_suggestions_name_their_folder() {
        let suggestions = [suggestion("d1", Some("Home/Bills")), suggestion("d2", None)];
        let state = present_inbox(Some(&catalog(true, None)), &suggestions, true, Vec::new());
        let notes: Vec<_> = state.rows.iter().map(|row| row.suggestion_note.as_str()).collect();
        assert_eq!(notes, vec![" · AI suggestion waiting", " · AI suggests Home/Bills"]);
        assert!(state.rows.iter().all(|row| row.has_suggestion && row.analyzable));
    }

    #[test]
    fn folders_to_file_into_leave_out_the_inbox_and_are_sorted_ignoring_case() {
        let state = present_inbox(Some(&catalog(true, None)), &[], true, Vec::new());
        assert_eq!(state.folders, vec!["archive", "Home", "Home/Bills", "Inboxes"]);
    }

    #[test]
    fn the_path_template_prefills_the_folder() {
        let state = present_inbox(Some(&catalog(true, Some("Filed/{kind}"))), &[], true, Vec::new());
        assert!(state.rows.iter().all(|row| row.prefill.as_deref().is_some_and(|folder| folder.starts_with("Filed/"))));
    }

    #[test]
    fn recent_targets_put_the_newest_first_without_repeats_and_keep_five() {
        let recent: Vec<String> = ["a", "b", "c", "d", "e"].map(str::to_owned).to_vec();
        assert_eq!(with_recent(&recent, "c"), vec!["c", "a", "b", "d", "e"]);
        assert_eq!(with_recent(&recent, "f"), vec!["f", "a", "b", "c", "d"]);
    }
}
