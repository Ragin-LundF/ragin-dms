use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

use dms_application::port::export::{ExportBundle, ExportTarget, ExportWriter};
use dms_application::port::storage::{StorageError, StorageFailure, StorageResult};
use dms_domain::document::ContentHash;
use dms_domain::library::{FileNamePolicy, LibraryFolder};
use indexmap::IndexMap;
use zip::write::FileOptions;
use zip::{AesMode, CompressionMethod, ZipArchive, ZipWriter};

use crate::export::index_files;
use crate::fs::{OrIoError, atomic_files, content_hasher, failure, path_resolver, root_of};

const CSV: &str = "index.csv";
const HTML: &str = "index.html";
const MISSING_ITEMS: &str = "Missing items.csv";

/// Writes exports outside the library: a new folder with hash-verified copies, or one ZIP file (optionally AES-256
/// encrypted) whose entries are read back and compared with the originals. A folder or file that already exists is
/// never replaced; the export takes the next free name. A failed export leaves no half-written ZIP behind.
#[derive(Debug, Clone, Copy, Default)]
pub struct FileExportWriter;

impl ExportWriter for FileExportWriter {
    fn is_inside(&self, directory: &str, folders: &[LibraryFolder]) -> bool {
        let Some(target) = normalized(Path::new(directory)) else { return true };
        folders.iter().any(|folder| {
            normalized(Path::new(folder.location.value()))
                .is_some_and(|root| target.starts_with(&root) || same_file(&target, &root))
        })
    }

    fn write(&self, source: &LibraryFolder, bundle: &ExportBundle, target: &ExportTarget) -> StorageResult<String> {
        let root = root_of(source)?;
        let directory = PathBuf::from(&target.directory);
        if !directory.is_dir() {
            return Err(StorageError::with_detail(StorageFailure::NotFound, target.directory.clone()));
        }
        let name = FileNamePolicy::sanitize(&target.name);
        if target.zip {
            let zip = ZipContent { bundle, extra: &[], password: None };
            write_zip(&root, &zip, &free(&directory, &format!("{name}.zip")).or_io()?)
        } else {
            write_folder(&root, bundle, &free(&directory, &name).or_io()?)
        }
    }

    fn write_text(&self, directory: &str, name: &str, text: &str) -> StorageResult<String> {
        self.write_file(directory, name, text.as_bytes())
    }

    fn write_file(&self, directory: &str, name: &str, content: &[u8]) -> StorageResult<String> {
        let parent = PathBuf::from(directory);
        if !parent.is_dir() {
            return Err(StorageError::with_detail(StorageFailure::NotFound, directory));
        }
        let file = free(&parent, &FileNamePolicy::sanitize(name)).or_io()?;
        atomic_files::write(&file, content).or_io()?;
        Ok(file.to_string_lossy().into_owned())
    }

    fn write_encrypted_zip(
        &self,
        source: &LibraryFolder,
        bundle: &ExportBundle,
        extra: &[(String, Vec<u8>)],
        target: &ExportTarget,
        password: &str,
    ) -> StorageResult<String> {
        let root = root_of(source)?;
        let directory = PathBuf::from(&target.directory);
        if !directory.is_dir() {
            return Err(StorageError::with_detail(StorageFailure::NotFound, target.directory.clone()));
        }
        let bundle = ExportBundle { index: None, ..bundle.clone() };
        let zip = ZipContent { bundle: &bundle, extra, password: Some(password) };
        let name = FileNamePolicy::sanitize(&target.name);
        write_zip(&root, &zip, &free(&directory, &format!("{name}.zip")).or_io()?)
    }
}

/// What goes into one ZIP file: the bundle, extra files given as their content, and the password of an encrypted
/// one.
struct ZipContent<'a> {
    bundle: &'a ExportBundle,
    extra: &'a [(String, Vec<u8>)],
    password: Option<&'a str>,
}

fn write_folder(root: &Path, bundle: &ExportBundle, folder: &Path) -> StorageResult<String> {
    fs::create_dir_all(folder).or_io()?;
    for file in &bundle.files {
        atomic_files::write(&folder.join(&file.target), &file.content).or_io()?;
    }
    for entry in &bundle.entries {
        let file = path_resolver::existing(root, &entry.source)
            .ok_or_else(|| StorageError::with_detail(StorageFailure::NotFound, entry.source.value()))?;
        atomic_files::copy(&file, &folder.join(&entry.target))?;
    }
    if let Some(rows) = &bundle.index {
        atomic_files::write(&folder.join(CSV), &index_files::csv(bundle, rows)).or_io()?;
        atomic_files::write(&folder.join(HTML), &index_files::html(bundle, rows)).or_io()?;
    }
    if let Some(items) = &bundle.missing_items {
        atomic_files::write(&folder.join(MISSING_ITEMS), &index_files::missing_items_csv(items)).or_io()?;
    }
    Ok(folder.to_string_lossy().into_owned())
}

fn write_zip(root: &Path, zip: &ZipContent<'_>, file: &Path) -> StorageResult<String> {
    let temp = atomic_files::temp_file_for(file);
    let result = write_zip_through(root, zip, file, &temp);
    if temp.exists() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn write_zip_through(root: &Path, zip: &ZipContent<'_>, file: &Path, temp: &Path) -> StorageResult<String> {
    let mut sources = IndexMap::new();
    for entry in &zip.bundle.entries {
        let source = path_resolver::existing(root, &entry.source)
            .ok_or_else(|| StorageError::with_detail(StorageFailure::NotFound, entry.source.value()))?;
        sources.insert(entry.target.clone(), source);
    }
    write_entries(zip, &sources, temp)
        .map_err(|error| StorageError::with_detail(StorageFailure::IoError, error.to_string()))?;
    if !verified(temp, &sources, zip).or_io()? {
        return Err(failure(StorageFailure::VerificationFailed));
    }
    atomic_files::move_without_replacing(temp, file)
        .map_err(|error| StorageError { reason: StorageFailure::IoError, ..error })?;
    Ok(file.to_string_lossy().into_owned())
}

fn write_entries(zip: &ZipContent<'_>, sources: &IndexMap<String, PathBuf>, temp: &Path) -> zip::result::ZipResult<()> {
    let mut writer = ZipWriter::new(fs::OpenOptions::new().write(true).create_new(true).open(temp)?);
    let plain = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);
    let options = match zip.password {
        Some(password) => plain.with_aes_encryption(AesMode::Aes256, password),
        None => plain,
    };
    for file in &zip.bundle.files {
        writer.start_file(file.target.as_str(), options)?;
        writer.write_all(&file.content)?;
    }
    for (name, source) in sources {
        writer.start_file(name.as_str(), options)?;
        io::copy(&mut File::open(source)?, &mut writer)?;
    }
    for (name, content) in zip.extra {
        writer.start_file(name.as_str(), options)?;
        writer.write_all(content)?;
    }
    if let Some(rows) = &zip.bundle.index {
        writer.start_file(CSV, options)?;
        writer.write_all(&index_files::csv(zip.bundle, rows))?;
        writer.start_file(HTML, options)?;
        writer.write_all(&index_files::html(zip.bundle, rows))?;
    }
    if let Some(items) = &zip.bundle.missing_items {
        writer.start_file(MISSING_ITEMS, options)?;
        writer.write_all(&index_files::missing_items_csv(items))?;
    }
    writer.finish()?.sync_all()?;
    Ok(())
}

/// Reads every entry back (decrypting it with the password) and compares it with the original.
fn verified(zip_file: &Path, sources: &IndexMap<String, PathBuf>, zip: &ZipContent<'_>) -> io::Result<bool> {
    let mut archive = ZipArchive::new(File::open(zip_file)?).map_err(io::Error::other)?;
    let mut entry_hash = |name: &str| -> io::Result<Option<ContentHash>> {
        let entry = match zip.password {
            Some(password) => archive.by_name_decrypt(name, password.as_bytes()),
            None => archive.by_name(name),
        };
        let Ok(mut entry) = entry else { return Ok(None) };
        content_hasher::hash_reader(&mut entry).map(Some)
    };
    for (name, source) in sources {
        if entry_hash(name)? != Some(content_hasher::hash(source)?) {
            return Ok(false);
        }
    }
    for (name, content) in zip.extra {
        if entry_hash(name)? != Some(content_hasher::hash_reader(&mut content.as_slice())?) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `name` in `directory`, or the first free `name (n)`; nothing existing is ever replaced.
fn free(directory: &Path, name: &str) -> io::Result<PathBuf> {
    let existing: Vec<String> = fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
        .collect::<io::Result<_>>()?;
    Ok(directory.join(FileNamePolicy::next_free_name(name, &existing)))
}

/// `Path.toAbsolutePath().normalize()`.
fn normalized(path: &Path) -> Option<PathBuf> {
    let absolute = std::path::absolute(path).ok()?;
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Some(normalized)
}

fn same_file(first: &Path, second: &Path) -> bool {
    match (fs::canonicalize(first), fs::canonicalize(second)) {
        (Ok(first), Ok(second)) => first == second,
        _ => false,
    }
}
