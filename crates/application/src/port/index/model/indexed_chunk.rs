/// One passage of a document's text, from character [`Self::start`] to [`Self::end`], with its embedding
/// [`Self::vector`].
#[derive(Debug, Clone, PartialEq)]
pub struct IndexedChunk {
    pub start: usize,
    pub end: usize,
    pub vector: Vec<f32>,
}
