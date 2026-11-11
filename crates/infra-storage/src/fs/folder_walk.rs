//! Walks a directory of a library folder for every folder and file below it, skipping what a scan skips.

use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use dms_application::port::storage::FolderEntries;
use dms_domain::library::RelativePath;
use indexmap::IndexSet;

use crate::fs::path_resolver;
use crate::scan::IgnoredEntries;

/// Every folder and file below `directory`; links are not followed and count as files, as in Java's
/// `Files.walkFileTree`.
///
/// # Errors
/// Fails when a directory cannot be listed.
pub fn entries(root: &Path, directory: &Path) -> io::Result<FolderEntries> {
    let mut entries = FolderEntries::default();
    let mut pending = VecDeque::from([directory.to_path_buf()]);
    while let Some(current) = pending.pop_front() {
        for child in sorted_children(&current)? {
            if is_ignored(root, &child) {
                continue;
            }
            if fs::symlink_metadata(&child)?.is_dir() {
                entries.folders.insert(relative_of(root, &child)?);
                pending.push_back(child);
            } else {
                entries.files.insert(relative_of(root, &child)?);
            }
        }
    }
    Ok(entries)
}

/// Every folder below `root`, skipping what a scan skips; only directories are followed, files are not read.
///
/// # Errors
/// Fails when a directory cannot be listed.
pub fn folders(root: &Path) -> io::Result<IndexSet<RelativePath>> {
    let mut folders = IndexSet::new();
    let mut pending = VecDeque::from([root.to_path_buf()]);
    while let Some(current) = pending.pop_front() {
        for child in sorted_children(&current)? {
            if fs::symlink_metadata(&child)?.is_dir() && !is_ignored(root, &child) {
                folders.insert(relative_of(root, &child)?);
                pending.push_back(child);
            }
        }
    }
    Ok(folders)
}

fn sorted_children(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut children =
        fs::read_dir(directory)?.map(|entry| entry.map(|entry| entry.path())).collect::<io::Result<Vec<_>>>()?;
    children.sort();
    Ok(children)
}

fn is_ignored(root: &Path, entry: &Path) -> bool {
    IgnoredEntries::is_ignored(&path_resolver::name_of(entry), entry.parent() == Some(root))
}

fn relative_of(root: &Path, entry: &Path) -> io::Result<RelativePath> {
    let relative = entry.strip_prefix(root).map_err(io::Error::other)?;
    let segments: Vec<String> =
        relative.components().map(|segment| path_resolver::name_of(Path::new(segment.as_os_str()))).collect();
    RelativePath::from_segments(&segments).map_err(io::Error::other)
}
