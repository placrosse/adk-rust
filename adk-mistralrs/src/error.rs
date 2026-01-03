//! Error types for adk-mistralrs.

use thiserror::Error;

/// Errors that can occur when using mistral.rs models.
#[derive(Debug, Error)]
pub enum MistralRsError {
    /// Model loading failed
    #[error("Model loading failed: {0}")]
    ModelLoad(String),

    /// Model file not found
    #[error("Model not found at path: {path}")]
    ModelNotFound { path: String },

    /// Unsupported model architecture
    #[error("Unsupported architecture: {0}")]
    UnsupportedArchitecture(String),

    /// Requested device is not available
    #[error("Device not available: {device}")]
    DeviceNotAvailable { device: String },

    /// Out of memory during model loading or inference
    #[error("Out of memory: {details}. Try reducing context length or enabling ISQ quantization.")]
    OutOfMemory { details: String },

    /// Inference failed
    #[error("Inference failed: {0}")]
    Inference(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Image processing failed
    #[error("Image processing failed: {0}")]
    ImageProcessing(String),

    /// Audio processing failed
    #[error("Audio processing failed: {0}")]
    AudioProcessing(String),

    /// Tool conversion failed
    #[error("Tool conversion failed: {0}")]
    ToolConversion(String),

    /// Chat template error
    #[error("Chat template error: {0}")]
    ChatTemplate(String),

    /// Adapter loading failed
    #[error("Adapter loading failed: {0}")]
    AdapterLoad(String),

    /// MCP client error
    #[error("MCP client error: {0}")]
    McpClient(String),

    /// Generic error wrapper
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result type alias for MistralRsError
pub type Result<T> = std::result::Result<T, MistralRsError>;
