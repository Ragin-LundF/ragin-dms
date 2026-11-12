use std::collections::HashMap;

use dms_application::port::export::{ExportBundle, ExportTarget, ExportWriter};
use dms_application::port::storage::StorageResult;
use dms_domain::library::LibraryFolder;

use crate::support::Shared;

/// An encrypted ZIP as written: the bundle, the extra files, the target and the password.
pub type EncryptedZip = (ExportBundle, Vec<(String, Vec<u8>)>, ExportTarget, String);

/// Records what would be written; every library folder location counts as inside the library.
#[derive(Default)]
pub struct FakeExportWriter {
    pub written: Shared<Vec<(ExportBundle, ExportTarget)>>,
    pub texts: Shared<HashMap<String, String>>,
    pub files: Shared<HashMap<String, Vec<u8>>>,
    pub encrypted: Shared<Vec<EncryptedZip>>,
}

impl ExportWriter for FakeExportWriter {
    fn is_inside(&self, directory: &str, folders: &[LibraryFolder]) -> bool {
        folders.iter().any(|folder| {
            let location = folder.location.value();
            directory == location || directory.starts_with(&format!("{location}/"))
        })
    }

    fn write(&self, _source: &LibraryFolder, bundle: &ExportBundle, target: &ExportTarget) -> StorageResult<String> {
        self.written.update(|written| written.push((bundle.clone(), target.clone())));
        Ok(format!("{}/{}", target.directory, target.name))
    }

    fn write_text(&self, directory: &str, name: &str, text: &str) -> StorageResult<String> {
        let location = format!("{directory}/{name}");
        self.texts.update(|texts| texts.insert(location.clone(), text.to_owned()));
        Ok(location)
    }

    fn write_file(&self, directory: &str, name: &str, content: &[u8]) -> StorageResult<String> {
        let location = format!("{directory}/{name}");
        self.files.update(|files| files.insert(location.clone(), content.to_vec()));
        Ok(location)
    }

    fn write_encrypted_zip(
        &self,
        _source: &LibraryFolder,
        bundle: &ExportBundle,
        extra: &[(String, Vec<u8>)],
        target: &ExportTarget,
        password: &str,
    ) -> StorageResult<String> {
        let record = (bundle.clone(), extra.to_vec(), target.clone(), password.to_owned());
        self.encrypted.update(|written| written.push(record));
        Ok(format!("{}/{}.zip", target.directory, target.name))
    }
}
