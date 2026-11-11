use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use dms_application::port::storage::StorageResult;

use crate::fs::{atomic_files, io_error};
use crate::yaml::{Node, YamlError, emit, parse_bytes};

/// One YAML file of the app data directory, written atomically.
#[derive(Debug, Clone)]
pub struct YamlFile {
    path: PathBuf,
}

impl YamlFile {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The stored document, or `None` if the file does not exist (yet, or any more) or holds just `null`.
    ///
    /// # Errors
    /// `IoError` when it cannot be read, `UnsupportedFormat` when it is empty or no YAML.
    pub fn read(&self) -> StorageResult<Option<Node>> {
        // Read first instead of asking whether it exists: the file can go between the question and the read.
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(io_error(&error)),
        };
        match parse_bytes(&bytes)? {
            None => Err(YamlError("No content to map due to end-of-input".to_owned()).into()),
            Some(Node::Null) => Ok(None),
            Some(node) => Ok(Some(node)),
        }
    }

    /// # Errors
    /// `IoError` when the file cannot be written.
    pub fn write(&self, node: &Node) -> StorageResult<()> {
        atomic_files::write(&self.path, emit(node).as_bytes()).map_err(|error| io_error(&error))
    }
}
