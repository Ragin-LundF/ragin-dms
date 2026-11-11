use std::fs;
use std::path::Path;

use dms_application::port::sidecar::{SidecarFile, SidecarKind, SidecarLayout, SidecarMove};
use dms_application::port::storage::{StorageFailure, StorageResult};
use dms_domain::document::DocumentId;
use dms_domain::library::LibraryFolder;

use crate::fs::{OrIoError, atomic_files, path_resolver, root_of};
use crate::index::FileVectorIndex;
use crate::sidecar::sidecar_paths;
use crate::text::FileTextStore;

/// `.dms-text/.layout`: written once a folder is completely in the subfolder layout.
const MARKER: &str = ".layout";
const LAYOUT_VERSION: &str = "2";

const KINDS: [SidecarKind; 2] = [SidecarKind::Text, SidecarKind::Index];

/// Moves the flat `.dms-text/<id>.txt` and `.dms-index/<id>.vec` files of version 1.x into their subfolders
/// (see [`sidecar_paths`]), by rename and never replacing a file.
#[derive(Debug, Clone, Copy, Default)]
pub struct FileSidecarLayout;

impl FileSidecarLayout {
    fn location(kind: SidecarKind) -> (&'static str, &'static str) {
        match kind {
            SidecarKind::Text => (FileTextStore::TEXT_FOLDER, ".txt"),
            SidecarKind::Index => (FileVectorIndex::INDEX_FOLDER, ".vec"),
        }
    }

    fn flat_files_of(root: &Path, kind: SidecarKind) -> StorageResult<Vec<SidecarFile>> {
        let (hidden_folder, suffix) = Self::location(kind);
        let directory = root.join(hidden_folder);
        if !directory.is_dir() {
            return Ok(Vec::new());
        }
        let mut files = Vec::new();
        for entry in fs::read_dir(&directory).or_io()? {
            let entry = entry.or_io()?;
            if !entry.file_type().or_io()?.is_file() {
                continue;
            }
            let name = entry.file_name();
            let Some(stem) = name.to_str().and_then(|name| name.strip_suffix(suffix)) else { continue };
            let Ok(document_id) = DocumentId::new(stem) else { continue };
            // Only names the path rule gives back: skips hidden and temporary files and ids that are no file name.
            if sidecar_paths::flat_file(root, hidden_folder, &document_id, suffix) == Some(entry.path()) {
                files.push(SidecarFile { kind, document_id });
            }
        }
        files.sort_by(|first, second| first.document_id.value().cmp(second.document_id.value()));
        Ok(files)
    }
}

impl SidecarLayout for FileSidecarLayout {
    fn is_current(&self, folder: &LibraryFolder) -> bool {
        path_resolver::root(folder).is_some_and(|root| {
            fs::read_to_string(root.join(FileTextStore::TEXT_FOLDER).join(MARKER))
                .is_ok_and(|version| version.trim() == LAYOUT_VERSION)
        })
    }

    fn flat_files(&self, folder: &LibraryFolder) -> StorageResult<Vec<SidecarFile>> {
        let root = root_of(folder)?;
        let mut files = Vec::new();
        for kind in KINDS {
            files.extend(Self::flat_files_of(&root, kind)?);
        }
        Ok(files)
    }

    fn move_into_subfolder(&self, folder: &LibraryFolder, file: &SidecarFile) -> StorageResult<SidecarMove> {
        let root = root_of(folder)?;
        let (hidden_folder, suffix) = Self::location(file.kind);
        let invalid = || sidecar_paths::invalid_id(&file.document_id);
        let source = sidecar_paths::flat_file(&root, hidden_folder, &file.document_id, suffix).ok_or_else(invalid)?;
        let target =
            sidecar_paths::subfolder_file(&root, hidden_folder, &file.document_id, suffix).ok_or_else(invalid)?;
        if let Some(subfolder) = target.parent() {
            fs::create_dir_all(subfolder).or_io()?;
        }
        match atomic_files::move_without_replacing(&source, &target) {
            Ok(()) => Ok(SidecarMove::Moved),
            Err(error) if error.reason == StorageFailure::AlreadyExists => {
                let identical = fs::read(&source).or_io()? == fs::read(&target).or_io()?;
                Ok(SidecarMove::KeptExisting { identical })
            }
            Err(error) => Err(error),
        }
    }

    fn mark_current(&self, folder: &LibraryFolder) -> StorageResult<()> {
        let root = root_of(folder)?;
        let marker = root.join(FileTextStore::TEXT_FOLDER).join(MARKER);
        atomic_files::write(&marker, format!("{LAYOUT_VERSION}\n").as_bytes()).or_io()
    }
}
