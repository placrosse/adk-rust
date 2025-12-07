//! # adk-model
//!
//! LLM model integrations for ADK (Gemini, OpenAI, Azure, etc.).
//!
//! ## Overview
//!
//! This crate provides LLM implementations for ADK agents. Currently supports:
//!
//! - [`GeminiModel`] - Google's Gemini models (2.0 Flash, Pro, etc.)
//! - [`OpenAIClient`] - OpenAI models (GPT-4o, GPT-4o-mini, etc.)
//! - [`AzureOpenAIClient`] - Azure OpenAI Service
//! - [`MockLlm`] - Mock LLM for testing
//!
//! ## Quick Start
//!
//! ### Gemini
//!
//! ```rust,no_run
//! use adk_model::GeminiModel;
//! use std::sync::Arc;
//!
//! let api_key = std::env::var("GOOGLE_API_KEY").unwrap();
//! let model = GeminiModel::new(&api_key, "gemini-2.0-flash-exp").unwrap();
//! ```
//!
//! ### OpenAI
//!
//! ```rust,ignore
//! use adk_model::openai::{OpenAIClient, OpenAIConfig};
//!
//! let model = OpenAIClient::new(OpenAIConfig::new(
//!     std::env::var("OPENAI_API_KEY").unwrap(),
//!     "gpt-4o-mini",
//! )).unwrap();
//! ```
//!
//! ## Supported Models
//!
//! ### Gemini
//! | Model | Description |
//! |-------|-------------|
//! | `gemini-2.0-flash-exp` | Fast, efficient model (recommended) |
//! | `gemini-1.5-pro` | Most capable model |
//! | `gemini-1.5-flash` | Balanced speed/capability |
//!
//! ### OpenAI
//! | Model | Description |
//! |-------|-------------|
//! | `gpt-4o` | Most capable model |
//! | `gpt-4o-mini` | Fast, cost-effective |
//! | `gpt-4-turbo` | Previous generation flagship |
//!
//! ## Features
//!
//! - Async streaming with backpressure
//! - Tool/function calling support
//! - Multimodal input (text, images, audio, video, PDF)
//! - Generation configuration (temperature, top_p, etc.)
//! - OpenAI-compatible APIs (Ollama, vLLM, etc.)

#[cfg(feature = "gemini")]
pub mod gemini;
pub mod mock;
#[cfg(feature = "openai")]
pub mod openai;

#[cfg(feature = "gemini")]
pub use gemini::GeminiModel;
pub use mock::MockLlm;
#[cfg(feature = "openai")]
pub use openai::{AzureOpenAIClient, OpenAIClient};
