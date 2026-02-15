//! In-memory vector store using cosine similarity.
//!
//! This module provides [`InMemoryVectorStore`], a zero-dependency vector store
//! backed by a `HashMap` protected by a `tokio::sync::RwLock`. It is suitable
//! for development, testing, and small-scale use cases.

use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::document::{Chunk, SearchResult};
use crate::error::{RagError, Result};
use crate::vectorstore::VectorStore;

/// An in-memory vector store using cosine similarity for search.
///
/// Collections are stored as nested `HashMap`s: collection name → chunk ID → chunk.
/// All operations are async-safe via `tokio::sync::RwLock`.
///
/// # Example
///
/// ```rust,ignore
/// use adk_rag::{InMemoryVectorStore, VectorStore};
///
/// let store = InMemoryVectorStore::new();
/// store.create_collection("docs", 384).await?;
/// ```
#[derive(Debug, Default)]
pub struct InMemoryVectorStore {
    collections: RwLock<HashMap<String, HashMap<String, Chunk>>>,
}

impl InMemoryVectorStore {
    /// Create a new empty in-memory vector store.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Compute cosine similarity between two vectors.
///
/// Both vectors are L2-normalized before computing the dot product.
/// Returns 0.0 if either vector has zero magnitude.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn create_collection(&self, name: &str, _dimensions: usize) -> Result<()> {
        let mut collections = self.collections.write().await;
        collections.entry(name.to_string()).or_default();
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let mut collections = self.collections.write().await;
        collections.remove(name);
        Ok(())
    }

    async fn upsert(&self, collection: &str, chunks: &[Chunk]) -> Result<()> {
        let mut collections = self.collections.write().await;
        let store = collections.get_mut(collection).ok_or_else(|| RagError::VectorStoreError {
            backend: "InMemory".to_string(),
            message: format!("collection '{collection}' does not exist"),
        })?;
        for chunk in chunks {
            store.insert(chunk.id.clone(), chunk.clone());
        }
        Ok(())
    }

    async fn delete(&self, collection: &str, ids: &[&str]) -> Result<()> {
        let mut collections = self.collections.write().await;
        let store = collections.get_mut(collection).ok_or_else(|| RagError::VectorStoreError {
            backend: "InMemory".to_string(),
            message: format!("collection '{collection}' does not exist"),
        })?;
        for id in ids {
            store.remove(*id);
        }
        Ok(())
    }

    async fn search(
        &self,
        collection: &str,
        embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        let collections = self.collections.read().await;
        let store = collections.get(collection).ok_or_else(|| RagError::VectorStoreError {
            backend: "InMemory".to_string(),
            message: format!("collection '{collection}' does not exist"),
        })?;

        let mut scored: Vec<SearchResult> = store
            .values()
            .map(|chunk| {
                let score = cosine_similarity(&chunk.embedding, embedding);
                SearchResult { chunk: chunk.clone(), score }
            })
            .collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        Ok(scored)
    }
}
