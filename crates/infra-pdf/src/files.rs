use std::fmt::Write;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::error::PdfError;

/// The prefix of the tool's unfinished writes; the library scanner skips such files.
pub const TEMPORARY_PREFIX: &str = ".dms-tmp-";

/// Size in bytes and SHA-256 (lower-case hex) of a written file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFacts {
    pub size: u64,
    pub content_hash: String,
}

/// Writes a new file at [`target`], which must not exist yet: [`write`] fills a `.dms-tmp-` file next to
/// it, which is then moved into place without ever replacing anything. The temporary file never stays.
pub(crate) fn write_new(
    target: &Path,
    write: impl FnOnce(&Path) -> Result<(), PdfError>,
) -> Result<FileFacts, PdfError> {
    if is_taken(target) {
        return Err(PdfError::AlreadyExists);
    }
    create_parent(target)?;
    let temp = temp_for(target);
    let result = write(&temp).and_then(|()| place(&temp, target));
    // The temp file is gone after a successful move; after a failure it is removed here.
    let _ = fs::remove_file(&temp);
    result
}

/// Moves a finished [`temp`] file to [`target`], which must not exist yet, not even with another case
/// or Unicode normalization.
pub(crate) fn place(temp: &Path, target: &Path) -> Result<FileFacts, PdfError> {
    if is_taken(target) {
        return Err(PdfError::AlreadyExists);
    }
    create_parent(target)?;
    move_without_replacing(temp, target)?;
    facts_of(target)
}

/// Moves [`temp`] to [`target`] unless something is there. A hard link fails atomically on an existing
/// target, which a plain rename does not (it replaces on Unix and, through Rust, on Windows too). File
/// systems without hard links (some network shares) fall back to check-then-rename.
pub(crate) fn move_without_replacing(temp: &Path, target: &Path) -> Result<(), PdfError> {
    match fs::hard_link(temp, target) {
        Ok(()) => {
            fs::remove_file(temp)?;
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Err(PdfError::AlreadyExists),
        Err(_) => {
            if fs::symlink_metadata(target).is_ok() {
                return Err(PdfError::AlreadyExists);
            }
            fs::rename(temp, target)?;
            Ok(())
        }
    }
}

pub(crate) fn temp_for(target: &Path) -> PathBuf {
    target.with_file_name(format!("{TEMPORARY_PREFIX}{}.pdf", uuid::Uuid::new_v4()))
}

/// Whether an entry next to [`target`] has its name, ignoring case and Unicode normalization.
fn is_taken(target: &Path) -> bool {
    let (Some(directory), Some(name)) = (target.parent(), target.file_name()) else {
        return false;
    };
    let wanted = comparison_key(&name.to_string_lossy());
    let Ok(entries) = fs::read_dir(if directory.as_os_str().is_empty() { Path::new(".") } else { directory }) else {
        return false;
    };
    entries.flatten().any(|entry| comparison_key(&entry.file_name().to_string_lossy()) == wanted)
}

fn comparison_key(name: &str) -> String {
    name.nfc().collect::<String>().to_lowercase()
}

fn create_parent(target: &Path) -> Result<(), PdfError> {
    if let Some(parent) = target.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub(crate) fn facts_of(file: &Path) -> Result<FileFacts, PdfError> {
    let mut input = fs::File::open(file)?;
    let mut digest = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    let mut size = 0_u64;
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
        size += read as u64;
    }
    let content_hash = digest.finalize().iter().fold(String::new(), |mut hex, byte| {
        let _ = write!(hex, "{byte:02x}");
        hex
    });
    Ok(FileFacts { size, content_hash })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_that_appears_before_the_move_is_not_replaced() {
        let root = tempfile::tempdir().expect("temp dir");
        let target = root.path().join("scan.pdf");
        fs::write(&target, "arrived meanwhile").expect("write target");
        let temp = temp_for(&target);
        fs::write(&temp, "written").expect("write temp");

        let result = move_without_replacing(&temp, &target);

        assert!(matches!(result, Err(PdfError::AlreadyExists)), "{result:?}");
        assert_eq!(fs::read_to_string(&target).expect("read"), "arrived meanwhile");
    }

    #[test]
    fn saving_onto_a_name_that_differs_only_by_case_or_normalization_fails_and_keeps_the_existing_file() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::write(root.path().join("SCAN.pdf"), "existing").expect("write");
        fs::write(root.path().join("Mu\u{308}ller.pdf"), "decomposed").expect("write");

        let by_case = write_new(&root.path().join("scan.pdf"), |temp| Ok(fs::write(temp, "new")?));
        let by_normalization = write_new(&root.path().join("m\u{fc}ller.pdf"), |temp| Ok(fs::write(temp, "new")?));

        assert!(matches!(by_case, Err(PdfError::AlreadyExists)));
        assert!(matches!(by_normalization, Err(PdfError::AlreadyExists)));
        assert_eq!(fs::read_to_string(root.path().join("SCAN.pdf")).expect("read"), "existing");
        assert_eq!(fs::read_dir(root.path()).expect("list").count(), 2);
    }

    #[test]
    fn a_written_file_is_placed_under_its_name_with_its_facts_and_leaves_no_temporary_file() {
        let root = tempfile::tempdir().expect("temp dir");
        let target = root.path().join("out/copy.pdf");

        let facts = write_new(&target, |temp| Ok(fs::write(temp, "abc")?)).expect("written");

        assert_eq!(facts.size, 3);
        assert_eq!(facts.content_hash, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        let names: Vec<_> =
            fs::read_dir(root.path().join("out")).expect("list").flatten().map(|e| e.file_name()).collect();
        assert_eq!(names, vec![std::ffi::OsString::from("copy.pdf")]);
    }

    #[test]
    fn a_failed_write_leaves_nothing_behind() {
        let root = tempfile::tempdir().expect("temp dir");

        let result = write_new(&root.path().join("new.pdf"), |temp| {
            fs::write(temp, "half")?;
            Err(PdfError::unsupported("broken"))
        });

        assert!(matches!(result, Err(PdfError::UnsupportedFormat(_))));
        assert_eq!(fs::read_dir(root.path()).expect("list").count(), 0);
    }
}
