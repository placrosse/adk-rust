//! Error types for the `adk-rag` crate.

use thiserror::Error;

/// Errors that can occur in RAG operations.
#[derive(Debug, Error)]
pub enum RagError {
    /// An error occurred during embedding generation.
    #[error("Embedding error ({provider}): {message}")]
    EmbeddingError {
        /// The embedding provider that produced the error.
        provider: String,
        /// A description of the failure.
        message: String,
    },

    /// An error occurred in the vector store backend.
    #[error("Vector store error ({backend}): {message}")]
    VectorStoreError {
        /// The vector store backend that produced the error.
        backend: String,
        /// A description of the failure.
        message: String,
    },

    /// An error occurred during document chunking.
    #[error("Chunking error: {0}")]
    ChunkingError(String),

    /// An error occurred during result reranking.
    #[error("Reranker error ({reranker}): {message}")]
    RerankerError {
        /// The reranker that produced the error.
        reranker: String,
        /// A description of the failure.
        message: String,
    },

    /// A configuration validation error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// An error in the RAG pipeline orchestration.
    #[error("Pipeline error: {0}")]
    PipelineError(String),

    /// An error propagated from `adk-core`.
    #[error(transparent)]
    AdkError(#[from] adk_core::AdkError),
}

/// A convenience result type for RAG operations.
pub type Result<T> = std::result::Result<T, RagError>;
