use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use dms_application::port::storage::{StorageFailure, StorageResult};
use dms_domain::change::FileFacts;
use dms_domain::document::{ContentHash, Document};
use dms_domain::library::RelativePath;
use dms_domain::text;
use indexmap::IndexMap;

use crate::yaml::{Fields, MapWriter, Node, YamlError, YamlFile};

/// Size, modification time and hash of every file at the last scan of one folder, kept on this device in
/// `scan-cache/<folderId>.yaml`. It lets a scan hash only files that are new or changed, so files a sync client
/// keeps only in the cloud are not downloaded by every scan.
pub struct ScanCache {
    file: YamlFile,
    entries: HashMap<String, Entry>,
}

/// What the last scan saw of one file; `content_hash` is absent when the file was trusted without hashing.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    size: i64,
    modified: i64,
    content_hash: Option<String>,
}

impl ScanCache {
    /// The cache in `path`; an unreadable cache starts empty, it only costs hashing time.
    ///
    /// # Errors
    /// Fails when the file exists but cannot be read.
    pub fn load(path: PathBuf) -> StorageResult<Self> {
        let file = YamlFile::new(path);
        let entries = match file.read() {
            Ok(node) => node.map(|node| entries_of(&node)).transpose().unwrap_or_default().unwrap_or_default(),
            Err(error) if error.reason == StorageFailure::UnsupportedFormat => HashMap::new(),
            Err(error) => return Err(error),
        };
        Ok(Self { file, entries })
    }

    /// The facts of one file. Hashes it when it is unknown to the catalog, differs from the catalog in size, or
    /// changed since the last scan; an unchanged catalogued file is trusted.
    ///
    /// # Errors
    /// Fails when hashing fails.
    pub fn facts<E>(
        &mut self,
        path: &RelativePath,
        size: u64,
        modified: i64,
        known: Option<&Document>,
        hash: impl FnOnce() -> Result<ContentHash, E>,
    ) -> Result<FileFacts, E> {
        let size_value = i64::try_from(size).unwrap_or(i64::MAX);
        let cached = self.entries.get(path.value());
        let unchanged = cached.is_some_and(|cached| cached.size == size_value && cached.modified == modified);
        let cached_hash = cached.and_then(|cached| cached.content_hash.as_deref()).map(ContentHash::from_hex);
        let content_hash = match cached_hash {
            Some(Ok(hash)) if unchanged => Some(hash),
            None if unchanged => None,
            // A damaged hash in the cache is read again; the Kotlin version failed the scan here.
            _ if unchanged || known.is_none_or(|known| known.size != size) || cached.is_some() => Some(hash()?),
            _ => None,
        };
        let content_hash_text = content_hash.as_ref().map(|hash| hash.value().to_owned());
        self.entries
            .insert(path.value().to_owned(), Entry { size: size_value, modified, content_hash: content_hash_text });
        Ok(FileFacts { size, content_hash })
    }

    /// Saves what is known so far, in the middle of a scan.
    ///
    /// # Errors
    /// Fails when the file cannot be written.
    pub fn checkpoint(&self) -> StorageResult<()> {
        self.file.write(&self.to_node())
    }

    /// Saves the cache, forgetting files that no longer exist.
    ///
    /// # Errors
    /// Fails when the file cannot be written.
    pub fn save<'a>(&mut self, present: impl IntoIterator<Item = &'a RelativePath>) -> StorageResult<()> {
        let keep: HashSet<&str> = present.into_iter().map(RelativePath::value).collect();
        self.entries.retain(|path, _| keep.contains(path.as_str()));
        self.checkpoint()
    }

    fn to_node(&self) -> Node {
        let mut paths: Vec<&String> = self.entries.keys().collect();
        paths.sort_by(|first, second| text::compare(first, second));
        let entries: IndexMap<String, Node> = paths
            .into_iter()
            .map(|path| {
                let entry = &self.entries[path];
                let node = MapWriter::new()
                    .int("size", entry.size)
                    .int("modified", entry.modified)
                    .opt_text("contentHash", entry.content_hash.as_deref())
                    .build();
                (path.clone(), node)
            })
            .collect();
        MapWriter::new().node("entries", Node::Map(entries)).build()
    }
}

fn entries_of(node: &Node) -> Result<HashMap<String, Entry>, YamlError> {
    Fields::of(node)?
        .entries("entries")?
        .into_iter()
        .map(|(path, entry)| {
            let fields = Fields::of(entry)?;
            let entry = Entry {
                size: fields.i64("size")?,
                modified: fields.i64("modified")?,
                content_hash: fields.opt_text("contentHash")?.map(str::to_owned),
            };
            Ok((path.to_owned(), entry))
        })
        .collect()
}
