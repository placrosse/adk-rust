//! MistralRsModel - the main model provider implementing the Llm trait.

use std::path::Path;
use std::sync::Arc;

use adk_core::{Content, LlmRequest, LlmResponse, LlmResponseStream, Llm, Part, FinishReason, UsageMetadata};
use async_trait::async_trait;
use futures::stream;
use mistralrs::{
    TextModelBuilder, IsqType, PagedAttentionMetaBuilder,
    TextMessages, TextMessageRole, Response,
};
use tracing::{debug, info, instrument};

use crate::config::{
    MistralRsConfig, ModelSource, QuantizationLevel,
};
use crate::error::{MistralRsError, Result};

/// mistral.rs model provider for ADK.
///
/// This struct wraps a mistral.rs model instance and implements the ADK `Llm` trait,
/// allowing it to be used with ADK agents and workflows.
///
/// # Example
///
/// ```rust,ignore
/// use adk_mistralrs::{MistralRsModel, MistralRsConfig, ModelSource};
///
/// let model = MistralRsModel::from_hf("microsoft/Phi-3.5-mini-instruct").await?;
/// ```
pub struct MistralRsModel {
    /// The underlying mistral.rs model instance
    model: Arc<mistralrs::Model>,
    /// Model name for identification
    name: String,
    /// Configuration used to create this model
    config: MistralRsConfig,
}

impl MistralRsModel {
    /// Create a new model from configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration specifying model source, architecture, and options
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = MistralRsConfig::builder()
    ///     .model_source(ModelSource::huggingface("microsoft/Phi-3.5-mini-instruct"))
    ///     .build();
    /// let model = MistralRsModel::new(config).await?;
    /// ```
    #[instrument(skip(config), fields(model_source = ?config.model_source))]
    pub async fn new(config: MistralRsConfig) -> Result<Self> {
        let model_id = match &config.model_source {
            ModelSource::HuggingFace(id) => id.clone(),
            ModelSource::Local(path) => path.display().to_string(),
            ModelSource::Gguf(path) => path.display().to_string(),
            ModelSource::Uqff(path) => path.display().to_string(),
        };

        info!("Loading mistral.rs model: {}", model_id);

        let mut builder = TextModelBuilder::new(model_id.clone());

        // Apply ISQ quantization if configured
        if let Some(isq) = &config.isq {
            let isq_type = quantization_level_to_isq(isq.level);
            builder = builder.with_isq(isq_type);
            debug!("ISQ quantization enabled: {:?}", isq.level);
        }

        // Apply PagedAttention if configured
        if config.paged_attention {
            builder = builder
                .with_paged_attn(|| PagedAttentionMetaBuilder::default().build())
                .map_err(|e| MistralRsError::ModelLoad(e.to_string()))?;
            debug!("PagedAttention enabled");
        }

        // Enable logging
        builder = builder.with_logging();

        // Build the model
        let model = builder
            .build()
            .await
            .map_err(|e| MistralRsError::ModelLoad(e.to_string()))?;

        info!("Model loaded successfully: {}", model_id);

        Ok(Self {
            model: Arc::new(model),
            name: model_id,
            config,
        })
    }

    /// Create from HuggingFace model ID with defaults.
    ///
    /// # Arguments
    ///
    /// * `model_id` - HuggingFace model ID (e.g., "microsoft/Phi-3.5-mini-instruct")
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let model = MistralRsModel::from_hf("microsoft/Phi-3.5-mini-instruct").await?;
    /// ```
    pub async fn from_hf(model_id: &str) -> Result<Self> {
        let config = MistralRsConfig::builder()
            .model_source(ModelSource::huggingface(model_id))
            .build();
        Self::new(config).await
    }

    /// Create from local GGUF file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the GGUF model file
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let model = MistralRsModel::from_gguf("/path/to/model.gguf").await?;
    /// ```
    pub async fn from_gguf(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(MistralRsError::ModelNotFound {
                path: path.display().to_string(),
            });
        }

        let config = MistralRsConfig::builder()
            .model_source(ModelSource::gguf(path))
            .build();
        Self::new(config).await
    }

    /// Create with ISQ quantization.
    ///
    /// # Arguments
    ///
    /// * `config` - Base configuration
    /// * `level` - Quantization level to apply
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = MistralRsConfig::builder()
    ///     .model_source(ModelSource::huggingface("mistralai/Mistral-7B-v0.1"))
    ///     .build();
    /// let model = MistralRsModel::with_isq(config, QuantizationLevel::Q4_0).await?;
    /// ```
    pub async fn with_isq(mut config: MistralRsConfig, level: QuantizationLevel) -> Result<Self> {
        config.isq = Some(crate::config::IsqConfig::new(level));
        Self::new(config).await
    }

    /// Get the model configuration
    pub fn config(&self) -> &MistralRsConfig {
        &self.config
    }

    /// Convert ADK request to mistral.rs messages
    fn build_messages(&self, request: &LlmRequest) -> TextMessages {
        let mut messages = TextMessages::new();

        for content in &request.contents {
            let role = match content.role.as_str() {
                "user" => TextMessageRole::User,
                "model" | "assistant" => TextMessageRole::Assistant,
                "system" => TextMessageRole::System,
                _ => TextMessageRole::User, // Default to user for unknown roles
            };

            // Extract text from parts
            let text: String = content
                .parts
                .iter()
                .filter_map(|part| match part {
                    Part::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");

            if !text.is_empty() {
                messages = messages.add_message(role, text);
            }
        }

        messages
    }

    /// Convert mistral.rs response to ADK response
    fn convert_response(&self, response: &mistralrs::ChatCompletionResponse) -> LlmResponse {
        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .map(|text| {
                Content::new("model").with_text(text.clone())
            });

        let usage_metadata = Some(UsageMetadata {
            prompt_token_count: response.usage.prompt_tokens as i32,
            candidates_token_count: response.usage.completion_tokens as i32,
            total_token_count: response.usage.total_tokens as i32,
        });

        let finish_reason = response
            .choices
            .first()
            .map(|choice| {
                match choice.finish_reason.as_str() {
                    "stop" => FinishReason::Stop,
                    "length" => FinishReason::MaxTokens,
                    _ => FinishReason::Other,
                }
            });

        LlmResponse {
            content,
            usage_metadata,
            finish_reason,
            partial: false,
            turn_complete: true,
            interrupted: false,
            error_code: None,
            error_message: None,
        }
    }
}

#[async_trait]
impl Llm for MistralRsModel {
    fn name(&self) -> &str {
        &self.name
    }

    #[instrument(skip(self, request), fields(model = %self.name))]
    async fn generate_content(
        &self,
        request: LlmRequest,
        stream: bool,
    ) -> adk_core::Result<LlmResponseStream> {
        debug!("Generating content with {} messages", request.contents.len());

        let messages = self.build_messages(&request);

        if stream {
            // Streaming response
            let model = Arc::clone(&self.model);
            
            let response_stream = async_stream::stream! {
                use futures::StreamExt;
                
                let stream_result = model
                    .stream_chat_request(messages)
                    .await;
                
                match stream_result {
                    Ok(mut stream) => {
                        let mut accumulated_text = String::new();

                        while let Some(chunk) = stream.next().await {
                            match chunk {
                                Response::Chunk(chunk_response) => {
                                    if let Some(choice) = chunk_response.choices.first() {
                                        if let Some(content) = &choice.delta.content {
                                            accumulated_text.push_str(content);
                                            
                                            let response = LlmResponse {
                                                content: Some(Content::new("model").with_text(content.clone())),
                                                usage_metadata: None,
                                                finish_reason: None,
                                                partial: true,
                                                turn_complete: false,
                                                interrupted: false,
                                                error_code: None,
                                                error_message: None,
                                            };
                                            yield Ok(response);
                                        }
                                    }
                                }
                                Response::Done(final_response) => {
                                    let usage = Some(UsageMetadata {
                                        prompt_token_count: final_response.usage.prompt_tokens as i32,
                                        candidates_token_count: final_response.usage.completion_tokens as i32,
                                        total_token_count: final_response.usage.total_tokens as i32,
                                    });

                                    let response = LlmResponse {
                                        content: Some(Content::new("model").with_text(accumulated_text.clone())),
                                        usage_metadata: usage,
                                        finish_reason: Some(FinishReason::Stop),
                                        partial: false,
                                        turn_complete: true,
                                        interrupted: false,
                                        error_code: None,
                                        error_message: None,
                                    };
                                    yield Ok(response);
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(adk_core::AdkError::Model(e.to_string()));
                    }
                }
            };

            Ok(Box::pin(response_stream))
        } else {
            // Non-streaming response
            let response = self
                .model
                .send_chat_request(messages)
                .await
                .map_err(|e| adk_core::AdkError::Model(e.to_string()))?;

            let adk_response = self.convert_response(&response);
            Ok(Box::pin(stream::once(async { Ok(adk_response) })))
        }
    }
}

/// Convert QuantizationLevel to mistral.rs IsqType
fn quantization_level_to_isq(level: QuantizationLevel) -> IsqType {
    match level {
        QuantizationLevel::Q4_0 => IsqType::Q4_0,
        QuantizationLevel::Q4_1 => IsqType::Q4_1,
        QuantizationLevel::Q5_0 => IsqType::Q5_0,
        QuantizationLevel::Q5_1 => IsqType::Q5_1,
        QuantizationLevel::Q8_0 => IsqType::Q8_0,
        QuantizationLevel::Q8_1 => IsqType::Q8_1,
        QuantizationLevel::Q2K => IsqType::Q2K,
        QuantizationLevel::Q3K => IsqType::Q3K,
        QuantizationLevel::Q4K => IsqType::Q4K,
        QuantizationLevel::Q5K => IsqType::Q5K,
        QuantizationLevel::Q6K => IsqType::Q6K,
    }
}

impl std::fmt::Debug for MistralRsModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MistralRsModel")
            .field("name", &self.name)
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_source_display() {
        let hf = ModelSource::huggingface("test/model");
        match hf {
            ModelSource::HuggingFace(id) => assert_eq!(id, "test/model"),
            _ => panic!("Expected HuggingFace variant"),
        }
    }

    #[test]
    fn test_quantization_level_conversion() {
        // Test all quantization levels can be converted
        let levels = [
            QuantizationLevel::Q4_0,
            QuantizationLevel::Q4_1,
            QuantizationLevel::Q5_0,
            QuantizationLevel::Q5_1,
            QuantizationLevel::Q8_0,
            QuantizationLevel::Q8_1,
            QuantizationLevel::Q2K,
            QuantizationLevel::Q3K,
            QuantizationLevel::Q4K,
            QuantizationLevel::Q5K,
            QuantizationLevel::Q6K,
        ];

        for level in levels {
            let _ = quantization_level_to_isq(level);
        }
    }
}
