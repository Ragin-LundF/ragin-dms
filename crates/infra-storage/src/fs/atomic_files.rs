//! Writes that never leave a half-written file behind: everything goes to a `.dms-tmp-<uuid>` file in the target
//! directory first and is then moved into place in one step.

use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

use dms_application::port::storage::{StorageError, StorageFailure, StorageResult};
use dms_domain::change::FileFacts;
use dms_domain::library::TemporaryNames;

use crate::fs::content_hasher;
use crate::fs::path_resolver;
use crate::fs::storage_io::{OrIoError, failure};

/// Copies `source` to `target`, which must not exist, verifies the copy by hash and keeps the modification time.
/// An entry that appears at `target` meanwhile, also one whose name differs only by case, is never replaced: the
/// result is `ALREADY_EXISTS`.
///
/// # Errors
/// `VERIFICATION_FAILED` when the written copy differs, `ALREADY_EXISTS` as above, `IO_ERROR` otherwise.
pub fn copy(source: &Path, target: &Path) -> StorageResult<FileFacts> {
    create_parent(target).or_io()?;
    let temp = temp_file_for(target);
    let result = copy_through(source, target, &temp);
    remove_leftover(&temp);
    result
}

fn copy_through(source: &Path, target: &Path, temp: &Path) -> StorageResult<FileFacts> {
    let source_hash = content_hasher::copy_and_hash(source, temp).or_io()?;
    if content_hasher::hash(temp).or_io()? != source_hash {
        return Err(failure(StorageFailure::VerificationFailed));
    }
    let modified = fs::metadata(source).and_then(|metadata| metadata.modified()).or_io()?;
    OpenOptions::new().write(true).open(temp).and_then(|file| file.set_modified(modified)).or_io()?;
    // Hashing a large file takes long; check again for a clash that appeared meanwhile.
    let directory = target.parent().unwrap_or(Path::new("."));
    if path_resolver::clashing(directory, &file_name(target)).or_io()?.is_some() {
        return Err(failure(StorageFailure::AlreadyExists));
    }
    move_without_replacing(temp, target)?;
    let size = fs::metadata(target).or_io()?.len();
    Ok(FileFacts { size, content_hash: Some(source_hash) })
}

/// Writes `bytes` to `target`, replacing an existing file in one step.
///
/// # Errors
/// Fails when the file cannot be written; the temporary file is removed then.
pub fn write(target: &Path, bytes: &[u8]) -> io::Result<()> {
    create_parent(target)?;
    let temp = temp_file_for(target);
    let result = write_through(target, &temp, bytes);
    remove_leftover(&temp);
    result
}

fn write_through(target: &Path, temp: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    // A rename replaces the target in one step on every platform the app runs on.
    fs::rename(temp, target)
}

/// A sibling name for a temporary file or an intermediate rename.
pub fn temp_file_for(target: &Path) -> PathBuf {
    target.with_file_name(format!("{}{}", TemporaryNames::PREFIX, uuid::Uuid::new_v4()))
}

/// Moves `temp` to `target` unless something is there. Never a plain `rename(2)`, which replaces an existing
/// target without asking.
///
/// # Errors
/// `ALREADY_EXISTS` when the target exists, `IO_ERROR` otherwise.
pub fn move_without_replacing(temp: &Path, target: &Path) -> StorageResult<()> {
    rename_new(temp, target).map_err(|error| {
        if error.kind() == ErrorKind::AlreadyExists {
            StorageError::with_detail(StorageFailure::AlreadyExists, error.to_string())
        } else {
            StorageError::with_detail(StorageFailure::IoError, error.to_string())
        }
    })
}

/// Renames without ever replacing an existing target, atomically where the system offers it.
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn rename_new(source: &Path, target: &Path) -> io::Result<()> {
    use rustix::fs::{CWD, RenameFlags, renameat_with};
    use rustix::io::Errno;
    match renameat_with(CWD, source, CWD, target, RenameFlags::NOREPLACE) {
        // Some file systems and network shares do not know the flag.
        Err(error) if [Errno::INVAL, Errno::NOTSUP, Errno::OPNOTSUPP, Errno::NOSYS].contains(&error) => {
            checked_rename(source, target)
        }
        result => result.map_err(io::Error::from),
    }
}

/// shortcut: on Windows the check and the rename are two steps, as `Files.move` was in the Kotlin version;
/// `MoveFileExW` without `REPLACE_EXISTING` would close the gap but needs a Windows API crate.
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn rename_new(source: &Path, target: &Path) -> io::Result<()> {
    checked_rename(source, target)
}

fn checked_rename(source: &Path, target: &Path) -> io::Result<()> {
    if fs::symlink_metadata(target).is_ok() {
        return Err(io::Error::new(ErrorKind::AlreadyExists, format!("{} exists", target.display())));
    }
    fs::rename(source, target)
}

fn create_parent(target: &Path) -> io::Result<()> {
    target.parent().map_or(Ok(()), fs::create_dir_all)
}

fn remove_leftover(temp: &Path) {
    if fs::symlink_metadata(temp).is_ok() {
        let _ = fs::remove_file(temp);
    }
}

fn file_name(path: &Path) -> String {
    path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()
}
