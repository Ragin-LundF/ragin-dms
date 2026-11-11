//! Every YAML file the tool writes, in the exact text Jackson 3 wrote in the Kotlin version: parsed into a
//! [`Node`] tree with Jackson's JSON schema and written back by a port of its emitter.

mod emitter;
mod fields;
mod map_writer;
mod node;
mod parser;
mod scalar_style;
mod yaml_file;

use std::fmt;

use dms_application::port::storage::{StorageError, StorageFailure};

pub use emitter::emit;
pub use fields::Fields;
pub use map_writer::MapWriter;
pub use node::Node;
#[cfg(test)]
pub use parser::parse;
pub use parser::parse_bytes;
pub use yaml_file::YamlFile;

/// A file that is no YAML, or whose content does not have the expected shape; Jackson's `JacksonException`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlError(pub String);

impl fmt::Display for YamlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A value that breaks a domain rule makes the file unreadable, as the Kotlin `IllegalArgumentException` did.
impl From<dms_domain::DomainError> for YamlError {
    fn from(error: dms_domain::DomainError) -> Self {
        YamlError(error.to_string())
    }
}

impl From<YamlError> for StorageError {
    fn from(error: YamlError) -> Self {
        StorageError::with_detail(StorageFailure::UnsupportedFormat, error.0)
    }
}
