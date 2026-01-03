//! # adk-mistralrs
//!
//! Native [mistral.rs](https://github.com/EricLBuehler/mistral.rs) integration for ADK-Rust,
//! providing blazingly fast local LLM inference without external dependencies.
//!
//! > **Note:** This crate is NOT published to crates.io because mistral.rs depends on
//! > unpublished git dependencies. Add it via git dependency instead.
//!
//! ## Features
//!
//! - **Native Rust Integration**: Direct embedding of mistral.rs, no daemon required
//! - **ISQ (In-Situ Quantization)**: Quantize models on-the-fly at load time
//! - **PagedAttention**: Memory-efficient attention for longer contexts
//! - **Multi-Device Support**: CPU, CUDA, Metal acceleration
//! - **Tool Calling**: Full function calling support via ADK interface
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use adk_mistralrs::{MistralRsModel, MistralRsConfig, ModelSource};
//! use adk_core::Llm;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let model = MistralRsModel::from_hf("microsoft/Phi-3.5-mini-instruct").await?;
//!     // Use with ADK agents...
//!     Ok(())
//! }
//! ```

mod config;
mod client;
mod convert;
mod error;

pub use config::*;
pub use client::*;
pub use error::*;

// Re-export commonly used types
pub use adk_core::{Llm, LlmRequest, LlmResponse, LlmResponseStream};
