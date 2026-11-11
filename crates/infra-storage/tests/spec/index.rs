use std::fs;

use dms_application::port::index::{DocumentVectors, IndexedChunk, VectorIndex};
use dms_domain::document::{ContentHash, DocumentId};
use dms_infra_storage::FileVectorIndex;

use crate::support::{folder, hash_of, write};

fn document_id() -> DocumentId {
    DocumentId::new("doc-1").expect("id")
}

#[test]
fn an_entry_is_written_to_the_hidden_index_folder_and_read_back() {
    let temp = tempfile::tempdir().expect("temp dir");
    let vectors = DocumentVectors {
        document_id: document_id(),
        model: "sample-embed".to_owned(),
        dimensions: 3,
        content_hash: hash_of("alpha"),
        metadata_hash: "1a2b".to_owned(),
        chunks: vec![
            IndexedChunk { start: 0, end: 10, vector: vec![0.1, 0.2, 0.3] },
            IndexedChunk { start: 8, end: 20, vector: vec![-1.0, 0.0, 1.0] },
        ],
    };

    FileVectorIndex.write(&folder(temp.path()), &vectors).expect("written");
    let read = FileVectorIndex.read(&folder(temp.path()), &document_id()).expect("an entry");

    assert!(temp.path().join(".dms-index/doc-1.vec").is_file());
    assert_eq!(read, vectors);
    assert!(read.is_current("sample-embed", &vectors.content_hash, "1a2b"));
    assert!(!read.is_current("other", &vectors.content_hash, "1a2b"));
}

#[test]
fn a_missing_or_broken_entry_reads_as_none() {
    let temp = tempfile::tempdir().expect("temp dir");
    write(&temp.path().join(".dms-index/doc-1.vec"), "not an index");

    assert_eq!(FileVectorIndex.read(&folder(temp.path()), &document_id()), None);
    assert_eq!(FileVectorIndex.read(&folder(temp.path()), &DocumentId::new("other").expect("id")), None);
}

/// The bytes Java's `DataOutputStream` wrote: modified UTF-8 for `U+0000` and the emoji, big-endian floats.
#[test]
fn an_entry_has_the_bytes_the_kotlin_version_wrote_and_reads_them_back() {
    let temp = tempfile::tempdir().expect("temp dir");
    let golden = fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/jackson/doc-1.vec"))
        .expect("golden");
    let vectors = DocumentVectors {
        document_id: document_id(),
        model: "embed-ä\u{0}😀".to_owned(),
        dimensions: 3,
        content_hash: ContentHash::new("8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8")
            .expect("hash"),
        metadata_hash: "1a2b".to_owned(),
        chunks: vec![
            IndexedChunk { start: 0, end: 10, vector: vec![0.1, -0.2, 3.5e-8] },
            IndexedChunk { start: 8, end: 20, vector: vec![-1.0, 0.0, f32::MAX] },
        ],
    };

    FileVectorIndex.write(&folder(temp.path()), &vectors).expect("written");

    assert_eq!(fs::read(temp.path().join(".dms-index/doc-1.vec")).expect("written file"), golden);
    assert_eq!(FileVectorIndex.read(&folder(temp.path()), &document_id()), Some(vectors));
}

#[test]
fn a_damaged_count_reads_as_none_instead_of_reserving_memory() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut bytes = fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/jackson/doc-1.vec"))
        .expect("golden");
    let dimensions_at = 4 + 4 + 2 + 16;
    bytes[dimensions_at..dimensions_at + 4].copy_from_slice(&i32::MAX.to_be_bytes());
    fs::create_dir_all(temp.path().join(".dms-index")).expect("folder");
    fs::write(temp.path().join(".dms-index/doc-1.vec"), bytes).expect("file");

    assert_eq!(FileVectorIndex.read(&folder(temp.path()), &document_id()), None);
}
