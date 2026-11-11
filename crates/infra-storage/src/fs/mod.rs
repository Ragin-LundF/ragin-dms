//! File operations in one library folder: the one path mapping, atomic and verified writes, hashing.

pub mod atomic_files;
pub mod content_hasher;
mod file_system_document_storage;
pub mod folder_walk;
pub mod path_resolver;
mod storage_io;

pub use file_system_document_storage::{FileSystemDocumentStorage, root_of};
pub use storage_io::{OrIoError, failure, format_error, io_error};
