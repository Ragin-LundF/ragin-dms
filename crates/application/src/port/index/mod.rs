//! The embedding index of a library folder.

pub mod model;
mod vector_index;

pub use model::{DocumentVectors, IndexedChunk};
pub use vector_index::VectorIndex;
