use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use dms_application::port::storage::{LibraryScanner, ScanListener, StorageFailure, StorageResult};
use dms_domain::catalog::Catalog;
use dms_domain::change::{FileFacts, FolderSnapshot};
use dms_domain::document::Document;
use dms_domain::library::{LibraryFolder, RelativePath};
use dms_domain::text;
use indexmap::IndexMap;

use crate::fs::{OrIoError, content_hasher, failure, root_of};
use crate::scan::{CloudPlaceholders, IgnoredEntries, ScanCache};

/// Lists every file of one library folder with NFC paths. Symbolic links are not followed. The scan cache of each
/// folder lives in the cache directory, outside the library.
///
/// A scan first lists the folder, which is cheap, and then reads the files one by one, so the listener knows the
/// total. The cache is saved every `checkpoint_every` hashed files: a cancelled or crashed first scan does not
/// have to read those files again.
pub struct FileSystemLibraryScanner {
    cache_directory: PathBuf,
    checkpoint_every: usize,
}

/// A regular file found while listing a folder, before its content is looked at.
struct ListedFile {
    path: RelativePath,
    entry: PathBuf,
    size: u64,
    modified: i64,
}

struct Listing<'a> {
    known: &'a HashMap<&'a RelativePath, &'a Document>,
    files: Vec<ListedFile>,
    placeholders: IndexMap<RelativePath, FileFacts>,
}

impl FileSystemLibraryScanner {
    pub const DEFAULT_CHECKPOINT: usize = 100;

    pub fn new(cache_directory: impl Into<PathBuf>) -> Self {
        Self::with_checkpoint(cache_directory, Self::DEFAULT_CHECKPOINT)
    }

    pub fn with_checkpoint(cache_directory: impl Into<PathBuf>, checkpoint_every: usize) -> Self {
        Self { cache_directory: cache_directory.into(), checkpoint_every: checkpoint_every.max(1) }
    }
}

impl LibraryScanner for FileSystemLibraryScanner {
    fn scan(
        &self,
        folder: &LibraryFolder,
        catalog: &Catalog,
        listener: &dyn ScanListener,
    ) -> StorageResult<FolderSnapshot> {
        let root = root_of(folder)?;
        let mut cache = ScanCache::load(self.cache_directory.join(format!("{}.yaml", folder.id.value())))?;
        let known: HashMap<&RelativePath, &Document> =
            catalog.documents().values().map(|document| (&document.path, document)).collect();
        let mut listing = Listing { known: &known, files: Vec::new(), placeholders: IndexMap::new() };
        list(&root, &[], &mut listing)?;
        listener.counted(listing.files.len());
        let mut files = IndexMap::new();
        let mut since_checkpoint = 0;
        for (index, file) in listing.files.iter().enumerate() {
            if listener.is_cancelled() {
                return Err(failure(StorageFailure::Cancelled));
            }
            let mut hashed = false;
            let facts = cache.facts(&file.path, file.size, file.modified, known.get(&file.path).copied(), || {
                hashed = true;
                content_hasher::hash(&file.entry)
            });
            let facts = facts.or_io()?;
            if hashed {
                since_checkpoint += 1;
                if since_checkpoint >= self.checkpoint_every {
                    cache.checkpoint()?;
                    since_checkpoint = 0;
                }
            }
            listener.file_done(index + 1, &file.path, &facts, hashed);
            files.insert(file.path.clone(), facts);
        }
        // A downloaded file wins over a leftover placeholder of the same name.
        for (path, facts) in listing.placeholders {
            files.entry(path).or_insert(facts);
        }
        cache.save(files.keys())?;
        Ok(FolderSnapshot { folder_id: folder.id.clone(), files })
    }
}

/// Depth first, the entries of each directory sorted by their name on disk.
fn list(directory: &Path, segments: &[String], listing: &mut Listing<'_>) -> StorageResult<()> {
    let mut entries: Vec<PathBuf> = fs::read_dir(directory)
        .and_then(|entries| entries.map(|entry| entry.map(|entry| entry.path())).collect())
        .or_io()?;
    entries.sort_by(|first, second| text::compare(&raw_name(first), &raw_name(second)));
    for entry in entries {
        let name = crate::fs::path_resolver::name_of(&entry);
        if IgnoredEntries::is_ignored(&name, segments.is_empty()) {
            continue;
        }
        let metadata = fs::symlink_metadata(&entry).or_io()?;
        let child = [segments, std::slice::from_ref(&name)].concat();
        if metadata.is_dir() {
            list(&entry, &child, listing)?;
        } else if !metadata.is_file() {
        } else if let Some(real_name) = CloudPlaceholders::real_name_of(&name) {
            let path = relative_path(&[segments, &[real_name.to_owned()]].concat())?;
            let size = listing.known.get(&path).map_or(0, |document| document.size);
            listing.placeholders.insert(path, FileFacts { size, content_hash: None });
        } else {
            let modified = metadata.modified().map(epoch_millis).or_io()?;
            listing.files.push(ListedFile { path: relative_path(&child)?, entry, size: metadata.len(), modified });
        }
    }
    Ok(())
}

fn relative_path(segments: &[String]) -> StorageResult<RelativePath> {
    RelativePath::from_segments(segments).map_err(|error| crate::fs::format_error(&error))
}

fn raw_name(path: &Path) -> String {
    path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()
}

/// `FileTime.toMillis()`: milliseconds since the epoch, rounded down also before it.
fn epoch_millis(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_millis()).unwrap_or(i64::MAX),
        Err(before) => {
            let before = before.duration();
            i64::try_from(before.as_nanos().div_ceil(1_000_000)).map_or(i64::MIN, |millis| -millis)
        }
    }
}
