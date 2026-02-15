//! Embedding provider trait for generating vector embeddings from text.

use async_trait::async_trait;

use crate::error::Result;

/// A provider that generates vector embeddings from text input.
///
/// Implementations wrap specific embedding backends (Gemini, OpenAI, etc.)
/// behind a unified async interface. The default [`embed_batch`](EmbeddingProvider::embed_batch)
/// implementation calls [`embed`](EmbeddingProvider::embed) sequentially;
/// backends that support native batching should override it.
///
/// # Example
///
/// ```rust,ignore
/// use adk_rag::EmbeddingProvider;
///
/// let provider = MyEmbeddingProvider::new();
/// let embedding = provider.embed("hello world").await?;
/// assert_eq!(embedding.len(), provider.dimensions());
/// ```
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding vector for a single text input.
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embedding vectors for a batch of text inputs.
    ///
    /// The default implementation calls [`embed`](EmbeddingProvider::embed)
    /// sequentially for each input. Override this method if the backend
    /// supports native batch embedding for better throughput.
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    /// Return the dimensionality of embeddings produced by this provider.
    fn dimensions(&self) -> usize;
}
