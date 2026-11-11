//! The one rule for where a per-document file lives: `<hidden folder>/<xx>/<documentId><suffix>`, `xx` being the
//! first two characters of the document id in lower case, so no folder holds more than a few hundred files.
//! Version 1.x kept the files flat in the hidden folder (`<hidden folder>/<documentId><suffix>`); reads still fall
//! back to that place.

use std::path::{Path, PathBuf};

use dms_application::port::storage::{StorageError, StorageFailure};
use dms_domain::document::DocumentId;

/// The file in its subfolder, or `None` for an id that is no plain file name (it could point outside the folder).
pub fn subfolder_file(root: &Path, hidden_folder: &str, document_id: &DocumentId, suffix: &str) -> Option<PathBuf> {
    let name = file_name(document_id, suffix)?;
    Some(root.join(hidden_folder).join(subfolder(document_id)?).join(name))
}

/// The file in the flat layout of version 1.x.
pub fn flat_file(root: &Path, hidden_folder: &str, document_id: &DocumentId, suffix: &str) -> Option<PathBuf> {
    Some(root.join(hidden_folder).join(file_name(document_id, suffix)?))
}

/// The file in its subfolder if it is there, else the flat one if that is there.
pub fn existing_file(root: &Path, hidden_folder: &str, document_id: &DocumentId, suffix: &str) -> Option<PathBuf> {
    [subfolder_file(root, hidden_folder, document_id, suffix), flat_file(root, hidden_folder, document_id, suffix)]
        .into_iter()
        .flatten()
        .find(|file| file.is_file())
}

/// The error for an id [`subfolder_file`] refuses.
pub fn invalid_id(document_id: &DocumentId) -> StorageError {
    StorageError::with_detail(StorageFailure::IoError, format!("No valid file name: {}", document_id.value()))
}

fn subfolder(document_id: &DocumentId) -> Option<String> {
    let shard: String = document_id.value().chars().take(2).flat_map(char::to_lowercase).collect();
    shard.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_')).then_some(shard)
}

fn file_name(document_id: &DocumentId, suffix: &str) -> Option<String> {
    let id = document_id.value();
    let plain = !id.starts_with('.')
        && !id.chars().any(|character| matches!(character, '/' | '\\' | ':') || character.is_control());
    plain.then(|| format!("{id}{suffix}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(raw: &str) -> DocumentId {
        DocumentId::new(raw).expect("valid id")
    }

    #[test]
    fn a_file_lives_in_the_subfolder_of_the_first_two_characters_of_its_id_in_lower_case() {
        let root = Path::new("/library");

        let file = subfolder_file(root, ".dms-text", &id("3F2504E0-4F89-11D3-9A0C-0305E82C3301"), ".txt");

        assert_eq!(file, Some(PathBuf::from("/library/.dms-text/3f/3F2504E0-4F89-11D3-9A0C-0305E82C3301.txt")));
    }

    #[test]
    fn the_flat_file_is_where_version_1_kept_it() {
        let file = flat_file(Path::new("/library"), ".dms-index", &id("ab12"), ".vec");

        assert_eq!(file, Some(PathBuf::from("/library/.dms-index/ab12.vec")));
    }

    #[test]
    fn an_id_that_could_leave_the_folder_has_no_file() {
        for raw in ["../escape", "a/b", "ab\\c", "..", ".hidden", "c:x", "a\u{0}b"] {
            assert_eq!(subfolder_file(Path::new("/library"), ".dms-text", &id(raw), ".txt"), None, "{raw}");
            assert_eq!(flat_file(Path::new("/library"), ".dms-text", &id(raw), ".txt"), None, "{raw}");
        }
    }

    #[test]
    fn an_id_of_one_character_has_a_subfolder_of_one_character() {
        let file = subfolder_file(Path::new("/library"), ".dms-text", &id("A"), ".txt");

        assert_eq!(file, Some(PathBuf::from("/library/.dms-text/a/A.txt")));
    }

    #[test]
    fn the_subfolder_file_wins_over_the_flat_one_and_the_flat_one_is_the_fallback() {
        let root = tempfile::tempdir().expect("temp dir");
        let document = id("ab12");
        let flat = flat_file(root.path(), ".dms-text", &document, ".txt").expect("flat");
        let sharded = subfolder_file(root.path(), ".dms-text", &document, ".txt").expect("sharded");
        assert_eq!(existing_file(root.path(), ".dms-text", &document, ".txt"), None);

        std::fs::create_dir_all(sharded.parent().expect("parent")).expect("folders");
        std::fs::write(&flat, "old").expect("flat file");
        assert_eq!(existing_file(root.path(), ".dms-text", &document, ".txt"), Some(flat));

        std::fs::write(&sharded, "new").expect("sharded file");
        assert_eq!(existing_file(root.path(), ".dms-text", &document, ".txt"), Some(sharded));
    }
}
