//! Reranker trait for re-scoring search results.

use async_trait::async_trait;

use crate::document::SearchResult;
use crate::error::Result;

/// A reranker that re-scores and reorders search results.
///
/// Implementations can use cross-encoder models, LLM-based scoring, or
/// other strategies to improve precision beyond initial vector similarity.
#[async_trait]
pub trait Reranker: Send + Sync {
    /// Rerank search results given the original query.
    ///
    /// Returns results in a new order with potentially updated scores.
    async fn rerank(&self, query: &str, results: Vec<SearchResult>) -> Result<Vec<SearchResult>>;
}

/// A no-op reranker that returns results unchanged.
///
/// Useful as a default when no reranking is needed.
///
/// # Example
///
/// ```rust,ignore
/// use adk_rag::NoOpReranker;
///
/// let reranker = NoOpReranker;
/// let reranked = reranker.rerank("query", results).await?;
/// // reranked == results (same order, same scores)
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpReranker;

#[async_trait]
impl Reranker for NoOpReranker {
    async fn rerank(&self, _query: &str, results: Vec<SearchResult>) -> Result<Vec<SearchResult>> {
        Ok(results)
    }
}
