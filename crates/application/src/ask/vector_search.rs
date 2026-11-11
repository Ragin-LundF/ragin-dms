use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use dms_domain::document::Document;
use dms_domain::library::LibraryFolder;

use crate::ask::ScoredChunk;
use crate::port::index::{DocumentVectors, VectorIndex};

/// Brute-force cosine search over the index, in memory: the vectors of a document are read once per file version
/// and kept. Fast enough for a personal library of some thousand documents.
pub struct VectorSearch {
    index: Arc<dyn VectorIndex>,
    cache: Mutex<HashMap<String, DocumentVectors>>,
}

impl VectorSearch {
    pub fn new(index: Arc<dyn VectorIndex>) -> Self {
        Self { index, cache: Mutex::new(HashMap::new()) }
    }

    /// Every passage of `documents` scored against `query`; documents without a current entry are left out.
    pub fn score(
        &self,
        folder: &LibraryFolder,
        documents: &[Document],
        query: &[f32],
        model: &str,
    ) -> Vec<ScoredChunk> {
        documents
            .iter()
            .flat_map(|document| {
                let Some(vectors) = self.vectors_of(folder, document) else { return Vec::new() };
                if vectors.model != model || vectors.dimensions != query.len() {
                    return Vec::new();
                }
                vectors
                    .chunks
                    .iter()
                    .map(|chunk| ScoredChunk {
                        document_id: document.id.clone(),
                        start: chunk.start,
                        end: chunk.end,
                        score: Self::cosine(query, &chunk.vector),
                    })
                    .collect()
            })
            .collect()
    }

    /// The average of a document's vectors, for "related documents"; `None` without a current entry.
    pub fn centroid(&self, folder: &LibraryFolder, document: &Document) -> Option<(String, Vec<f32>)> {
        let vectors = self.vectors_of(folder, document).filter(|vectors| !vectors.chunks.is_empty())?;
        let mut sum = vec![0.0_f32; vectors.dimensions];
        for chunk in &vectors.chunks {
            for (total, value) in sum.iter_mut().zip(&chunk.vector) {
                *total += value;
            }
        }
        #[allow(clippy::cast_precision_loss)] // chunk counts stay far below f32 precision (at most 60)
        let count = vectors.chunks.len() as f32;
        Some((vectors.model, sum.into_iter().map(|total| total / count).collect()))
    }

    /// Products are taken in `f32` and summed in `f64`, as Kotlin did with `FloatArray`s.
    pub fn cosine(first: &[f32], second: &[f32]) -> f64 {
        let (mut dot, mut first_norm, mut second_norm) = (0.0_f64, 0.0_f64, 0.0_f64);
        for (a, b) in first.iter().zip(second) {
            dot += f64::from(a * b);
            first_norm += f64::from(a * a);
            second_norm += f64::from(b * b);
        }
        if first_norm == 0.0 || second_norm == 0.0 {
            return 0.0;
        }
        dot / (first_norm.sqrt() * second_norm.sqrt())
    }

    fn vectors_of(&self, folder: &LibraryFolder, document: &Document) -> Option<DocumentVectors> {
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
        let key = format!("{}:{}", document.id, document.content_hash.value());
        if let Some(vectors) = cache.get(&key) {
            return Some(vectors.clone());
        }
        let vectors =
            self.index.read(folder, &document.id).filter(|vectors| vectors.content_hash == document.content_hash)?;
        cache.insert(key, vectors.clone());
        Some(vectors)
    }
}
