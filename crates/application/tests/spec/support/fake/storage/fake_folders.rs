use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write;
use std::sync::Arc;

use chrono::NaiveDate;
use dms_application::port::storage::{
    CatalogLoad, CatalogRepository, DocumentStorage, ExternalFile, FolderEntries, LibraryScanner, ScanListener,
    StorageError, StorageFailure, StorageResult, TextStore,
};
use dms_domain::catalog::{Catalog, CatalogRevision};
use dms_domain::change::{FileFacts, FolderSnapshot};
use dms_domain::document::{ContentHash, DocumentId};
use dms_domain::library::{LibraryFolder, LibraryFolderId, RelativePath};
use dms_domain::text;
use indexmap::{IndexMap, IndexSet};
use sha2::{Digest, Sha256};

use crate::support::Shared;
use crate::support::test_data::path;

type BeforeSave = Box<dyn FnMut(&LibraryFolder) + Send>;

/// Every library folder of a test, in memory: file contents per path, empty folders, the catalog file with its
/// conflict copies, and `.dms-text`. Every call for a folder in `unreachable` fails.
#[derive(Default)]
pub struct FakeFolders {
    pub unreachable: Arc<Shared<HashSet<LibraryFolderId>>>,
    pub state: Shared<FolderState>,
    /// Runs before every catalog save, so a test can let another device write in between.
    before_save: Shared<Option<BeforeSave>>,
}

#[derive(Default)]
pub struct FolderState {
    pub trashed: Vec<(LibraryFolderId, RelativePath)>,
    pub files: HashMap<LibraryFolderId, IndexMap<RelativePath, String>>,
    pub directories: HashMap<LibraryFolderId, IndexSet<RelativePath>>,
    /// The catalog file of each folder.
    pub catalogs: HashMap<LibraryFolderId, Catalog>,
    /// Conflict copies of the catalog file, as a sync client creates them.
    pub conflict_copies: HashMap<LibraryFolderId, Vec<Catalog>>,
    /// Folders whose catalog file cannot be written, while everything else in them works.
    pub read_only_catalogs: HashSet<LibraryFolderId>,
    pub texts: HashMap<LibraryFolderId, HashMap<DocumentId, String>>,
    pub external: HashMap<ExternalFile, String>,
    /// Creation days the test set; files without one have none, like on file systems without them.
    pub creation_days: HashMap<RelativePath, NaiveDate>,
}

impl FolderState {
    fn files_of(&mut self, folder_id: &LibraryFolderId) -> &mut IndexMap<RelativePath, String> {
        self.files.entry(folder_id.clone()).or_default()
    }

    fn file_paths(&self, folder_id: &LibraryFolderId) -> Vec<RelativePath> {
        self.files.get(folder_id).map(|files| files.keys().cloned().collect()).unwrap_or_default()
    }

    fn directories_of(&self, folder_id: &LibraryFolderId) -> Vec<RelativePath> {
        self.directories.get(folder_id).map(|dirs| dirs.iter().cloned().collect()).unwrap_or_default()
    }

    fn content(&self, folder_id: &LibraryFolderId, path: &RelativePath) -> Option<String> {
        self.files.get(folder_id).and_then(|files| files.get(path)).cloned()
    }

    fn exists(&self, folder_id: &LibraryFolderId, path: &RelativePath) -> bool {
        self.content(folder_id, path).is_some() || self.is_directory(folder_id, path)
    }

    fn is_directory(&self, folder_id: &LibraryFolderId, path: &RelativePath) -> bool {
        let explicit = self.directories_of(folder_id);
        explicit.contains(path)
            || explicit.iter().any(|dir| dir.is_inside(path))
            || self.file_paths(folder_id).iter().any(|file| file.is_inside(path))
    }
}

impl FakeFolders {
    // --- Test setup and inspection

    pub fn put(&self, folder: &LibraryFolder, raw_path: &str, content: &str) {
        self.state.update(|state| state.files_of(&folder.id).insert(path(raw_path), content.to_owned()));
    }

    pub fn content(&self, folder: &LibraryFolder, raw_path: &str) -> Option<String> {
        self.state.lock().content(&folder.id, &path(raw_path))
    }

    pub fn paths(&self, folder: &LibraryFolder) -> BTreeSet<String> {
        self.state.lock().file_paths(&folder.id).iter().map(|path| path.value().to_owned()).collect()
    }

    pub fn has_directory(&self, folder: &LibraryFolder, raw_path: &str) -> bool {
        self.state.lock().is_directory(&folder.id, &path(raw_path))
    }

    pub fn text_in(&self, folder: &LibraryFolder, document_id: &DocumentId) -> Option<String> {
        self.state.lock().texts.get(&folder.id).and_then(|texts| texts.get(document_id)).cloned()
    }

    pub fn external(&self, location: &str, content: &str) -> ExternalFile {
        let file = ExternalFile::new(location).expect("valid location");
        self.state.update(|state| state.external.insert(file.clone(), content.to_owned()));
        file
    }

    pub fn catalog_in(&self, folder: &LibraryFolder) -> Option<Catalog> {
        self.state.lock().catalogs.get(&folder.id).cloned()
    }

    pub fn set_catalog(&self, folder: &LibraryFolder, catalog: Catalog) {
        self.state.update(|state| state.catalogs.insert(folder.id.clone(), catalog));
    }

    pub fn add_conflict_copy(&self, folder: &LibraryFolder, catalog: Catalog) {
        self.state.update(|state| state.conflict_copies.entry(folder.id.clone()).or_default().push(catalog));
    }

    pub fn conflict_copies_in(&self, folder: &LibraryFolder) -> Vec<Catalog> {
        self.state.lock().conflict_copies.get(&folder.id).cloned().unwrap_or_default()
    }

    pub fn trashed(&self) -> Vec<(LibraryFolderId, RelativePath)> {
        self.state.lock().trashed.clone()
    }

    pub fn set_creation_day(&self, raw_path: &str, day: NaiveDate) {
        self.state.update(|state| state.creation_days.insert(path(raw_path), day));
    }

    pub fn make_catalog_read_only(&self, folder: &LibraryFolder) {
        self.state.update(|state| state.read_only_catalogs.insert(folder.id.clone()));
    }

    pub fn make_unreachable(&self, folder: &LibraryFolder) {
        self.unreachable.update(|unreachable| unreachable.insert(folder.id.clone()));
    }

    pub fn make_reachable(&self, folder: &LibraryFolder) {
        self.unreachable.update(|unreachable| unreachable.remove(&folder.id));
    }

    pub fn set_before_save(&self, hook: impl FnMut(&LibraryFolder) + Send + 'static) {
        self.before_save.set(Some(Box::new(hook)));
    }

    /// The facts a scanner reports for `content`: its length and SHA-256.
    pub fn facts_of(content: &str) -> FileFacts {
        FileFacts { size: content.len() as u64, content_hash: Some(Self::hash_of(content)) }
    }

    pub fn hash_of(content: &str) -> ContentHash {
        let digest = Sha256::digest(content.as_bytes());
        let hex = digest.iter().fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        });
        ContentHash::new(hex).expect("valid hash")
    }

    // --- Helpers

    fn check_reachable(&self, folder: &LibraryFolder) -> StorageResult<()> {
        if self.unreachable.lock().contains(&folder.id) {
            return Err(failure(StorageFailure::FolderUnreachable));
        }
        Ok(())
    }

    fn run_before_save(&self, folder: &LibraryFolder) {
        // Taken out while it runs, so the hook may use this fake itself.
        let hook = self.before_save.update(Option::take);
        if let Some(mut hook) = hook {
            hook(folder);
            self.before_save.update(|slot| {
                if slot.is_none() {
                    *slot = Some(hook);
                }
            });
        }
    }
}

fn failure(reason: StorageFailure) -> StorageError {
    StorageError::new(reason)
}

impl DocumentStorage for FakeFolders {
    fn import_file(
        &self,
        folder: &LibraryFolder,
        source: &ExternalFile,
        target: &RelativePath,
    ) -> StorageResult<FileFacts> {
        self.check_reachable(folder)?;
        let mut state = self.state.lock();
        let content = state.external.get(source).cloned().ok_or_else(|| failure(StorageFailure::NotFound))?;
        if state.exists(&folder.id, target) {
            return Err(failure(StorageFailure::AlreadyExists));
        }
        state.files_of(&folder.id).insert(target.clone(), content.clone());
        Ok(Self::facts_of(&content))
    }

    fn copy_file(
        &self,
        from: &LibraryFolder,
        to: &LibraryFolder,
        path: &RelativePath,
        replace_existing: bool,
    ) -> StorageResult<FileFacts> {
        self.check_reachable(from)?;
        self.check_reachable(to)?;
        let mut state = self.state.lock();
        let content = state.content(&from.id, path).ok_or_else(|| failure(StorageFailure::NotFound))?;
        if state.content(&to.id, path).is_some() {
            if !replace_existing {
                return Err(failure(StorageFailure::AlreadyExists));
            }
            state.trashed.push((to.id.clone(), path.clone()));
        }
        state.files_of(&to.id).insert(path.clone(), content.clone());
        Ok(Self::facts_of(&content))
    }

    fn create_folder(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<()> {
        self.check_reachable(folder)?;
        let mut state = self.state.lock();
        if state.content(&folder.id, path).is_some() {
            return Err(failure(StorageFailure::AlreadyExists));
        }
        state.directories.entry(folder.id.clone()).or_default().insert(path.clone());
        Ok(())
    }

    fn move_entry(&self, folder: &LibraryFolder, from: &RelativePath, to: &RelativePath) -> StorageResult<()> {
        self.check_reachable(folder)?;
        let mut state = self.state.lock();
        let case_only =
            from.parent() == to.parent() && text::lowercase(from.file_name()) == text::lowercase(to.file_name());
        if state.exists(&folder.id, to) && !case_only {
            return Err(failure(StorageFailure::AlreadyExists));
        }
        if let Some(content) = state.files_of(&folder.id).shift_remove(from) {
            state.files_of(&folder.id).insert(to.clone(), content);
            return Ok(());
        }
        if !state.is_directory(&folder.id, from) {
            return Err(failure(StorageFailure::NotFound));
        }
        let files = state.files_of(&folder.id);
        let inside: Vec<RelativePath> = files.keys().filter(|path| path.is_inside(from)).cloned().collect();
        for path in inside {
            let content = files.shift_remove(&path).expect("listed just now");
            files.insert(path.relocate(from, to).expect("inside"), content);
        }
        if let Some(dirs) = state.directories.get_mut(&folder.id) {
            let moved: Vec<RelativePath> =
                dirs.iter().filter(|dir| *dir == from || dir.is_inside(from)).cloned().collect();
            dirs.retain(|dir| !moved.contains(dir));
            dirs.extend(moved.iter().map(|dir| dir.relocate(from, to).expect("inside")));
        }
        Ok(())
    }

    fn replace_with(
        &self,
        folder: &LibraryFolder,
        target: &RelativePath,
        replacement: &RelativePath,
    ) -> StorageResult<FileFacts> {
        self.check_reachable(folder)?;
        let mut state = self.state.lock();
        let content =
            state.files_of(&folder.id).shift_remove(replacement).ok_or_else(|| failure(StorageFailure::NotFound))?;
        if state.content(&folder.id, target).is_some() {
            state.trashed.push((folder.id.clone(), target.clone()));
        }
        state.files_of(&folder.id).insert(target.clone(), content.clone());
        Ok(Self::facts_of(&content))
    }

    fn move_to_trash(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<()> {
        self.check_reachable(folder)?;
        let mut state = self.state.lock();
        if state.files_of(&folder.id).shift_remove(path).is_some() {
            state.trashed.push((folder.id.clone(), path.clone()));
            return Ok(());
        }
        if !state.is_directory(&folder.id, path) {
            return Err(failure(StorageFailure::NotFound));
        }
        state.files_of(&folder.id).retain(|file, _| !file.is_inside(path));
        if let Some(dirs) = state.directories.get_mut(&folder.id) {
            dirs.retain(|dir| dir != path && !dir.is_inside(path));
        }
        state.trashed.push((folder.id.clone(), path.clone()));
        Ok(())
    }

    fn read_facts(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<FileFacts> {
        self.check_reachable(folder)?;
        let content = self.state.lock().content(&folder.id, path).ok_or_else(|| failure(StorageFailure::NotFound))?;
        Ok(Self::facts_of(&content))
    }

    fn same_content(&self, folder: &LibraryFolder, first: &RelativePath, second: &RelativePath) -> StorageResult<bool> {
        self.check_reachable(folder)?;
        let state = self.state.lock();
        let first_content = state.content(&folder.id, first).ok_or_else(|| failure(StorageFailure::NotFound))?;
        let second_content = state.content(&folder.id, second).ok_or_else(|| failure(StorageFailure::NotFound))?;
        Ok(first != second && first_content == second_content)
    }

    fn created_on(&self, _folder: &LibraryFolder, path: &RelativePath) -> Option<NaiveDate> {
        self.state.lock().creation_days.get(path).copied()
    }

    fn list_names(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<Vec<String>> {
        self.check_reachable(folder)?;
        let state = self.state.lock();
        if !path.is_root() && !state.is_directory(&folder.id, path) {
            return Err(failure(StorageFailure::NotFound));
        }
        let depth = path.segments().len();
        let entries: IndexSet<RelativePath> =
            state.file_paths(&folder.id).into_iter().chain(state.directories_of(&folder.id)).collect();
        let names: IndexSet<String> = entries
            .iter()
            .filter(|entry| entry.is_inside(path))
            .map(|entry| entry.segments()[depth].to_owned())
            .collect();
        Ok(names.into_iter().collect())
    }

    fn list_entries(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<FolderEntries> {
        self.check_reachable(folder)?;
        let state = self.state.lock();
        if !path.is_root() && !state.is_directory(&folder.id, path) {
            return Err(failure(StorageFailure::NotFound));
        }
        let below = |entry: &RelativePath| path.is_root() || entry.is_inside(path);
        let files: IndexSet<RelativePath> =
            state.file_paths(&folder.id).into_iter().filter(|file| below(file)).collect();
        let explicit: Vec<RelativePath> =
            state.directories_of(&folder.id).into_iter().filter(|dir| below(dir)).collect();
        let parents = files.iter().chain(explicit.iter()).flat_map(|entry| {
            std::iter::successors(entry.parent(), RelativePath::parent)
                .filter(|parent| !parent.is_root() && below(parent))
        });
        let folders: IndexSet<RelativePath> = explicit.iter().cloned().chain(parents).collect();
        Ok(FolderEntries { folders, files })
    }

    fn list_folders(&self, folder: &LibraryFolder) -> StorageResult<IndexSet<RelativePath>> {
        self.list_entries(folder, &RelativePath::root()).map(|entries| entries.folders)
    }
}

impl LibraryScanner for FakeFolders {
    fn scan(
        &self,
        folder: &LibraryFolder,
        _catalog: &Catalog,
        listener: &dyn ScanListener,
    ) -> StorageResult<FolderSnapshot> {
        self.check_reachable(folder)?;
        let facts: IndexMap<RelativePath, FileFacts> = self
            .state
            .lock()
            .files
            .get(&folder.id)
            .map(|files| files.iter().map(|(path, content)| (path.clone(), Self::facts_of(content))).collect())
            .unwrap_or_default();
        listener.counted(facts.len());
        for (index, (path, file_facts)) in facts.iter().enumerate() {
            listener.file_done(index + 1, path, file_facts, true);
        }
        Ok(FolderSnapshot { folder_id: folder.id.clone(), files: facts })
    }
}

impl CatalogRepository for FakeFolders {
    fn load(&self, folder: &LibraryFolder) -> CatalogLoad {
        if let Err(error) = self.check_reachable(folder) {
            return CatalogLoad::Failed(error);
        }
        let state = self.state.lock();
        let Some(catalog) = state.catalogs.get(&folder.id).cloned() else {
            return CatalogLoad::NotFound;
        };
        CatalogLoad::Found {
            catalog,
            conflict_copies: state.conflict_copies.get(&folder.id).cloned().unwrap_or_default(),
        }
    }

    fn save(&self, folder: &LibraryFolder, catalog: &Catalog, based_on: CatalogRevision) -> StorageResult<()> {
        self.check_reachable(folder)?;
        if self.state.lock().read_only_catalogs.contains(&folder.id) {
            return Err(failure(StorageFailure::IoError));
        }
        self.run_before_save(folder);
        let mut state = self.state.lock();
        if state.catalogs.get(&folder.id).is_some_and(|on_disk| on_disk.revision() > based_on) {
            return Err(failure(StorageFailure::CatalogChanged));
        }
        state.catalogs.insert(folder.id.clone(), catalog.clone());
        Ok(())
    }

    fn discard_conflict_copies(&self, folder: &LibraryFolder) -> StorageResult<()> {
        self.check_reachable(folder)?;
        self.state.update(|state| state.conflict_copies.remove(&folder.id));
        Ok(())
    }
}

impl TextStore for FakeFolders {
    fn write(&self, folder: &LibraryFolder, document_id: &DocumentId, text: &str) -> StorageResult<()> {
        self.check_reachable(folder)?;
        self.state.update(|state| {
            state.texts.entry(folder.id.clone()).or_default().insert(document_id.clone(), text.to_owned())
        });
        Ok(())
    }

    fn read(&self, folder: &LibraryFolder, document_id: &DocumentId) -> Option<String> {
        if self.unreachable.lock().contains(&folder.id) {
            return None;
        }
        self.text_in(folder, document_id)
    }
}
