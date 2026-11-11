//! Presenter and plan tests with synthetic catalogs. Never use real document names here.

mod attention;
mod plan;
mod stats;
mod tasks;

use chrono::{DateTime, Utc};
use dms_application::library::{Dashboard, FolderState};
use dms_domain::catalog::{Catalog, CatalogUpdate, LibraryId};
use dms_domain::change::NewFile;
use dms_domain::document::{ContentHash, Document, DocumentId};
use dms_domain::library::{FolderLocation, FolderRole, LibraryFolder, LibraryFolderId, RelativePath};
use dms_domain::task::OpenTask;
use dms_domain::taxonomy::{Category, CategoryId, TagName};

use super::{DashboardInput, present_dashboard};
use crate::features::dashboard::dto::{DashboardButtonDto, DashboardViewDto, TaskGroupDto, TaskGroupKind, TaskRowDto};

pub const PRIMARY: &str = "f-primary";
pub const SYNC: &str = "f-sync";

pub fn now() -> DateTime<Utc> {
    instant("2026-03-10T12:00:00Z")
}

pub fn instant(text: &str) -> DateTime<Utc> {
    text.parse().expect("valid instant")
}

pub fn path(raw: &str) -> RelativePath {
    RelativePath::of(raw).expect("valid path")
}

pub fn id(value: &str) -> DocumentId {
    DocumentId::new(value).expect("valid id")
}

pub fn folder_id(value: &str) -> LibraryFolderId {
    LibraryFolderId::new(value).expect("valid id")
}

pub fn hash(seed: u32) -> ContentHash {
    ContentHash::new(format!("{seed:064x}")).expect("valid hash")
}

/// A synthetic catalog, built change by change like the real one.
pub struct Library {
    catalog: Catalog,
}

impl Library {
    pub fn new() -> Self {
        Self { catalog: Catalog::empty(LibraryId::new("library-1").expect("valid id")) }
    }

    pub fn category(self, id: &str, name: &str) -> Self {
        let category = Category::new(CategoryId::new(id).expect("valid id"), name, None, None).expect("valid category");
        let update = self.catalog.add_category(category).expect("valid change");
        Self::applied(update)
    }

    /// A document added at `added` (RFC 3339) with its own content; `edit` sets what else it has.
    pub fn document(self, id_value: &str, raw_path: &str, added: &str, edit: impl FnOnce(&mut Document)) -> Self {
        let seed = u32::try_from(self.catalog.documents().len()).expect("small") + 1;
        self.document_with(id_value, raw_path, added, seed, edit)
    }

    /// A document with the content `seed`; documents with the same seed are copies of each other.
    pub fn document_with(
        self,
        id_value: &str,
        raw_path: &str,
        added: &str,
        seed: u32,
        edit: impl FnOnce(&mut Document),
    ) -> Self {
        let mut document =
            Document::new(id(id_value), path(raw_path), hash(seed), 2048, instant(added)).expect("valid");
        edit(&mut document);
        let update = self.catalog.add_document(document).expect("valid change");
        Self::applied(update)
    }

    pub fn inbox(self, raw_path: &str, ai_analysis: bool) -> Self {
        let mut settings = self.catalog.settings().clone();
        settings.layout.inbox.enabled = true;
        settings.layout.inbox.path = path(raw_path);
        settings.layout.inbox.ai_analysis = ai_analysis;
        let update = self.catalog.change_settings(settings).expect("valid change");
        Self::applied(update)
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    fn applied(update: CatalogUpdate) -> Self {
        match update {
            CatalogUpdate::Applied(catalog) => Self { catalog },
            CatalogUpdate::Rejected(reason) => panic!("expected an applied change, got {reason:?}"),
        }
    }
}

pub fn categorized(category: &str) -> impl FnOnce(&mut Document) {
    let category = CategoryId::new(category).expect("valid id");
    move |document| document.category_id = Some(category)
}

pub fn tagged(tags: &[&str]) -> impl FnOnce(&mut Document) {
    let names: Vec<TagName> = tags.iter().map(|tag| TagName::of(tag).expect("valid tag")).collect();
    move |document| document.tags.extend(names)
}

pub fn nothing(_: &mut Document) {}

pub fn folders() -> Vec<FolderState> {
    let folder = |id: &str, name: &str, location: &str, role| FolderState {
        folder: LibraryFolder::new(folder_id(id), name, FolderLocation::new(location).expect("valid"), role)
            .expect("valid folder"),
        reachable: Some(true),
        pending_operations: 0,
    };
    vec![
        folder(PRIMARY, "Main", "/Synthetic/Main", FolderRole::Primary),
        folder(SYNC, "Cloud", "/Synthetic/Cloud", FolderRole::Sync),
    ]
}

pub fn dashboard(tasks: Vec<OpenTask>) -> Dashboard {
    Dashboard {
        tasks,
        folders: folders(),
        document_count: 0,
        uncategorized_count: 0,
        recent_activity: Vec::new(),
        upcoming: Vec::new(),
    }
}

pub fn new_file(raw_path: &str, seed: Option<u32>, folders: &[&str]) -> NewFile {
    NewFile {
        path: path(raw_path),
        present_in: folders.iter().map(|id| folder_id(id)).collect(),
        size: 2048,
        content_hash: seed.map(hash),
    }
}

pub fn present(catalog: &Catalog, dashboard: Option<&Dashboard>) -> DashboardViewDto {
    present_dashboard(&DashboardInput {
        catalog,
        dashboard,
        foreign: &[],
        last_scan_at: None,
        ai_enabled: true,
        scan_count: None,
        now: now(),
        zone: &Utc,
    })
}

pub fn group(view: &DashboardViewDto, kind: TaskGroupKind) -> &TaskGroupDto {
    view.groups.iter().find(|group| group.group == kind).expect("the group is shown")
}

pub fn only_row(view: &DashboardViewDto, kind: TaskGroupKind) -> &TaskRowDto {
    let rows = &group(view, kind).rows;
    assert_eq!(rows.len(), 1, "one row in {kind:?}");
    &rows[0]
}

pub fn labels(buttons: &[DashboardButtonDto]) -> Vec<&str> {
    buttons.iter().map(|button| button.label.as_str()).collect()
}

pub fn named<'b>(buttons: &'b [DashboardButtonDto], label: &str) -> &'b DashboardButtonDto {
    buttons.iter().find(|button| button.label == label).expect("the button is there")
}
