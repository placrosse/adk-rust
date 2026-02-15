//! Data types for documents, chunks, and search results.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A source document containing text content and metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    /// Unique identifier for the document.
    pub id: String,
    /// The text content of the document.
    pub text: String,
    /// Key-value metadata associated with the document.
    pub metadata: HashMap<String, String>,
    /// Optional URI pointing to the original source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_uri: Option<String>,
}

/// A segment of a [`Document`] with its vector embedding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chunk {
    /// Unique identifier for the chunk.
    pub id: String,
    /// The text content of the chunk.
    pub text: String,
    /// The vector embedding for this chunk's text.
    pub embedding: Vec<f32>,
    /// Key-value metadata inherited from the parent document plus chunk-specific fields.
    pub metadata: HashMap<String, String>,
    /// The ID of the parent [`Document`].
    pub document_id: String,
}

/// A retrieved [`Chunk`] paired with a relevance score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The retrieved chunk.
    pub chunk: Chunk,
    /// The similarity score (higher is more relevant).
    pub score: f32,
}
