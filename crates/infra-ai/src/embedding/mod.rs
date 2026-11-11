//! Embeddings on the local server.

mod open_ai_embeddings;

pub use open_ai_embeddings::OpenAiEmbeddings;

#[cfg(test)]
mod tests;
