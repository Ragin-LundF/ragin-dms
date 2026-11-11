use std::sync::Arc;

use dms_domain::document::{Document, DocumentId};
use dms_domain::query::DocumentQuery;
use dms_domain::text;
use indexmap::{IndexMap, IndexSet};

use crate::ask::text_chunks::substring;
use crate::ask::{AskOutcome, Citation, DocumentTexts, ScoredChunk, TextChunks, VectorSearch};
use crate::common::LibrarySnapshot;
use crate::port::ai::{EmbeddingClient, EmbeddingResult};
use crate::port::assistant::{AssistantAnswer, LibraryAssistant, SourceChunk};
use crate::search::ContentMatcher;

const SOURCES: usize = 8;
const MIN_SIMILARITY: f64 = 0.3;
const KEYWORD_WEIGHT: f64 = 0.5;
const MIN_WORD: usize = 4;

/// Answers a question about the library. The documents in scope are searched two ways: by the words of the
/// question in their text and metadata, and by meaning through the index. The best passages go to the chat model,
/// which answers with citations `[n]`; they are mapped back to the documents. When nothing fits, the answer is an
/// honest "not found" without asking the AI.
pub struct AskLibrary {
    store: Arc<crate::library::CatalogStore>,
    search: Arc<ContentMatcher>,
    texts: Arc<DocumentTexts>,
    embeddings: Arc<dyn EmbeddingClient>,
    vectors: Arc<VectorSearch>,
    assistant: Arc<dyn LibraryAssistant>,
}

impl AskLibrary {
    pub fn new(
        store: Arc<crate::library::CatalogStore>,
        search: Arc<ContentMatcher>,
        texts: Arc<DocumentTexts>,
        embeddings: Arc<dyn EmbeddingClient>,
        vectors: Arc<VectorSearch>,
        assistant: Arc<dyn LibraryAssistant>,
    ) -> Self {
        Self { store, search, texts, embeddings, vectors, assistant }
    }

    /// `scope` is Kotlin's default `DocumentQuery::default()` (all documents) unless the user narrowed it.
    pub fn run(&self, question: &str, scope: &DocumentQuery) -> AskOutcome {
        let Some(library) = self.store.current() else {
            return AskOutcome::LibraryNotOpen;
        };
        let documents = self.search.matching(&library, scope);
        let semantic = self.semantic_chunks(&library, &documents, question);
        let lexical = self.keyword_chunks(&library, &documents, &words_of(question));
        let best: Vec<ScoredChunk> =
            merge(semantic.as_deref().unwrap_or_default(), &lexical).into_iter().take(SOURCES).collect();
        if best.is_empty() {
            return AskOutcome::NotFound;
        }
        let sources: Vec<SourceChunk> =
            best.iter().enumerate().map(|(index, chunk)| self.source_of(&library, chunk, index + 1)).collect();
        match self.assistant.answer(question, &sources) {
            AssistantAnswer::Failed { reason, detail } => AskOutcome::Failed { reason, detail },
            AssistantAnswer::Tags { .. } => AskOutcome::NotFound,
            AssistantAnswer::Text { text, model } => AskOutcome::Answered {
                citations: citations_of(&text, &best, &library),
                text,
                model,
                used_vectors: semantic.is_some(),
            },
        }
    }

    /// The passages closest in meaning, or `None` when the index or the embedding model is not there.
    fn semantic_chunks(
        &self,
        library: &LibrarySnapshot,
        documents: &[Document],
        question: &str,
    ) -> Option<Vec<ScoredChunk>> {
        let EmbeddingResult::Vectors { model, vectors } = self.embeddings.embed(&[question.to_owned()]) else {
            return None;
        };
        let vector = vectors.first()?;
        let scored = self.vectors.score(library.folders.primary(), documents, vector, &model);
        if scored.is_empty() {
            return None;
        }
        Some(scored.into_iter().filter(|chunk| chunk.score >= MIN_SIMILARITY).collect())
    }

    /// For each document with words of the question, its passage with the most of them.
    fn keyword_chunks(
        &self,
        library: &LibrarySnapshot,
        documents: &[Document],
        words: &IndexSet<String>,
    ) -> Vec<ScoredChunk> {
        if words.is_empty() {
            return Vec::new();
        }
        documents
            .iter()
            .filter_map(|document| {
                let header = text::lowercase(&TextChunks::header(document, &library.catalog));
                let document_text = self.texts.of(library.folders.primary(), document);
                #[allow(clippy::cast_precision_loss)] // word counts of a question
                let scored = TextChunks::of(&document_text).into_iter().map(|chunk| {
                    let haystack = format!("{header}\n{}", text::lowercase(&chunk.text));
                    let hits = words.iter().filter(|word| haystack.contains(word.as_str())).count();
                    ScoredChunk {
                        document_id: document.id.clone(),
                        start: chunk.start,
                        end: chunk.end,
                        score: hits as f64 / words.len() as f64,
                    }
                });
                // Kotlin's maxByOrNull keeps the first of equal scores.
                scored.filter(|chunk| chunk.score > 0.0).fold(None, |best: Option<ScoredChunk>, chunk| match best {
                    Some(best) if best.score >= chunk.score => Some(best),
                    _ => Some(chunk),
                })
            })
            .collect()
    }

    fn source_of(&self, library: &LibrarySnapshot, chunk: &ScoredChunk, number: usize) -> SourceChunk {
        let document = &library.catalog.documents()[&chunk.document_id];
        let document_text = self.texts.of(library.folders.primary(), document);
        let units: Vec<u16> = text::trim(&document_text).encode_utf16().collect();
        let start = chunk.start.min(units.len());
        let end = chunk.end.min(units.len());
        let passage = substring(&units, start, end.max(start));
        SourceChunk {
            number,
            title: title_of(document),
            text: format!("{}\n\n{passage}", TextChunks::header(document, &library.catalog)),
        }
    }
}

fn title_of(document: &Document) -> String {
    document.metadata.title.clone().unwrap_or_else(|| document.path.file_name().to_owned())
}

/// Both lists by score, a passage found both ways counting twice, one passage per place.
fn merge(semantic: &[ScoredChunk], lexical: &[ScoredChunk]) -> Vec<ScoredChunk> {
    let weighted = lexical.iter().map(|chunk| ScoredChunk { score: chunk.score * KEYWORD_WEIGHT, ..chunk.clone() });
    let mut combined: IndexMap<(DocumentId, usize, usize), ScoredChunk> = IndexMap::new();
    for chunk in semantic.iter().cloned().chain(weighted) {
        combined
            .entry((chunk.document_id.clone(), chunk.start, chunk.end))
            .and_modify(|found| found.score += chunk.score)
            .or_insert(chunk);
    }
    let mut combined: Vec<ScoredChunk> = combined.into_values().collect();
    combined.sort_by(|first, second| second.score.total_cmp(&first.score));
    combined
}

/// The documents the answer cites as `[n]`, each number once, in the order of the answer. Like Java's `\d`, only
/// ASCII digits count; a number too big for an `Int` is skipped.
fn citations_of(answer: &str, chunks: &[ScoredChunk], library: &LibrarySnapshot) -> Vec<Citation> {
    let mut numbers: IndexSet<usize> = IndexSet::new();
    let mut rest = answer;
    while let Some(open) = rest.find('[') {
        rest = &rest[open + 1..];
        let digits = rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        if digits > 0
            && rest[digits..].starts_with(']')
            && let Ok(number) = rest[..digits].parse::<i32>()
        {
            numbers.insert(usize::try_from(number).unwrap_or(0));
        }
    }
    numbers
        .into_iter()
        .filter_map(|number| {
            let chunk = chunks.get(number.checked_sub(1)?)?;
            let document = library.catalog.documents().get(&chunk.document_id)?;
            Some(Citation { number, document_id: document.id.clone(), title: title_of(document) })
        })
        .collect()
}

/// The words of at least four UTF-16 units. `char::is_alphabetic`/`is_numeric` stand in for Java's `\p{L}\p{N}`
/// (they also count letter numbers and a few combining marks as word characters).
fn words_of(question: &str) -> IndexSet<String> {
    text::lowercase(question)
        .split(|c: char| !(c.is_alphabetic() || c.is_numeric()))
        .filter(|word| text::utf16_len(word) >= MIN_WORD)
        .map(str::to_owned)
        .collect()
}
