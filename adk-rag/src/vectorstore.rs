//! Vector store trait for storing and searching vector embeddings.

use async_trait::async_trait;

use crate::document::{Chunk, SearchResult};
use crate::error::Result;

/// A storage backend for vector embeddings with similarity search.
///
/// Implementations manage named collections of [`Chunk`]s and support
/// upserting, deleting, and searching by vector similarity.
///
/// # Example
///
/// ```rust,ignore
/// use adk_rag::{VectorStore, InMemoryVectorStore};
///
/// let store = InMemoryVectorStore::new();
/// store.create_collection("docs", 384).await?;
/// store.upsert("docs", &chunks).await?;
/// let results = store.search("docs", &query_embedding, 5).await?;
/// ```
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Create a named collection. No-op if it already exists.
    async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()>;

    /// Delete a named collection and all its data.
    async fn delete_collection(&self, name: &str) -> Result<()>;

    /// Upsert chunks into a collection. Chunks must have embeddings set.
    async fn upsert(&self, collection: &str, chunks: &[Chunk]) -> Result<()>;

    /// Delete chunks by their IDs from a collection.
    async fn delete(&self, collection: &str, ids: &[&str]) -> Result<()>;

    /// Search for the `top_k` most similar chunks to the given embedding.
    ///
    /// Returns results ordered by descending similarity score.
    async fn search(
        &self,
        collection: &str,
        embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResult>>;
}
