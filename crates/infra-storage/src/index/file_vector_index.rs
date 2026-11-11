use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use dms_application::port::index::{DocumentVectors, IndexedChunk, VectorIndex};
use dms_application::port::storage::StorageResult;
use dms_domain::document::{ContentHash, DocumentId};
use dms_domain::library::LibraryFolder;

use crate::fs::{OrIoError, atomic_files, path_resolver, root_of};
use crate::index::java_data::{DataInput, DataOutput};

/// `.dms-index/<documentId>.vec`, as Java's `DataOutputStream` wrote it: int magic `0x444D5356`, int format 1, UTF
/// model, int dimensions, UTF content hash, UTF metadata hash, int chunk count, then per chunk int start, int end
/// and the float32 values. Derived data: a file that cannot be read counts as missing and is simply rebuilt.
#[derive(Debug, Clone, Copy, Default)]
pub struct FileVectorIndex;

const MAGIC: i32 = 0x444D_5356;
const FORMAT: i32 = 1;

impl FileVectorIndex {
    pub const INDEX_FOLDER: &'static str = ".dms-index";

    fn file_of(root: &Path, document_id: &DocumentId) -> PathBuf {
        root.join(Self::INDEX_FOLDER).join(format!("{}.vec", document_id.value()))
    }
}

impl VectorIndex for FileVectorIndex {
    fn write(&self, folder: &LibraryFolder, vectors: &DocumentVectors) -> StorageResult<()> {
        let root = root_of(folder)?;
        let bytes = encode(vectors).or_io()?;
        atomic_files::write(&Self::file_of(&root, &vectors.document_id), &bytes).or_io()
    }

    fn read(&self, folder: &LibraryFolder, document_id: &DocumentId) -> Option<DocumentVectors> {
        let file = Self::file_of(&path_resolver::root(folder)?, document_id);
        let bytes = file.is_file().then(|| fs::read(file).ok()).flatten()?;
        decode(document_id, &bytes)
    }
}

pub fn encode(vectors: &DocumentVectors) -> io::Result<Vec<u8>> {
    let mut output = DataOutput::default();
    output.write_int(MAGIC);
    output.write_int(FORMAT);
    output.write_utf(&vectors.model)?;
    output.write_int(to_int(vectors.dimensions)?);
    output.write_utf(vectors.content_hash.value())?;
    output.write_utf(&vectors.metadata_hash)?;
    output.write_int(to_int(vectors.chunks.len())?);
    for chunk in &vectors.chunks {
        output.write_int(to_int(chunk.start)?);
        output.write_int(to_int(chunk.end)?);
        chunk.vector.iter().for_each(|value| output.write_float(*value));
    }
    Ok(output.into_bytes())
}

pub fn decode(document_id: &DocumentId, bytes: &[u8]) -> Option<DocumentVectors> {
    let mut input = DataInput::new(bytes);
    if input.read_int()? != MAGIC || input.read_int()? != FORMAT {
        return None;
    }
    let model = input.read_utf()?;
    let dimensions = usize::try_from(input.read_int()?).ok()?;
    let content_hash = ContentHash::new(input.read_utf()?).ok()?;
    let metadata_hash = input.read_utf()?;
    let chunk_count = usize::try_from(input.read_int()?).ok()?;
    let mut chunks = Vec::new();
    for _ in 0..chunk_count {
        let start = usize::try_from(input.read_int()?).ok()?;
        let end = usize::try_from(input.read_int()?).ok()?;
        // A damaged count must not make us reserve memory the file cannot fill.
        if dimensions.checked_mul(4)? > input.remaining() {
            return None;
        }
        let vector = (0..dimensions).map(|_| input.read_float()).collect::<Option<Vec<f32>>>()?;
        chunks.push(IndexedChunk { start, end, vector });
    }
    let vectors =
        DocumentVectors { document_id: document_id.clone(), model, dimensions, content_hash, metadata_hash, chunks };
    vectors.validate().ok().map(|()| vectors)
}

fn to_int(value: usize) -> io::Result<i32> {
    i32::try_from(value).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "value too large for the index"))
}
