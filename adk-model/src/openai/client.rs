//! OpenAI client implementation.

use super::config::{AzureConfig, OpenAIConfig};
use super::convert;
use adk_core::{AdkError, Llm, LlmRequest};
use async_openai::{
    config::{AzureConfig as AsyncAzureConfig, OpenAIConfig as AsyncOpenAIConfig},
    types::CreateChatCompletionRequestArgs,
    Client,
};
use async_stream::try_stream;
use async_trait::async_trait;
use futures::StreamExt;

/// OpenAI client for standard OpenAI API and OpenAI-compatible APIs.
pub struct OpenAIClient {
    client: Client<AsyncOpenAIConfig>,
    model: String,
}

impl OpenAIClient {
    /// Create a new OpenAI client.
    pub fn new(config: OpenAIConfig) -> Result<Self, AdkError> {
        let mut openai_config = AsyncOpenAIConfig::new().with_api_key(&config.api_key);

        if let Some(org_id) = &config.organization_id {
            openai_config = openai_config.with_org_id(org_id);
        }

        if let Some(base_url) = &config.base_url {
            openai_config = openai_config.with_api_base(base_url);
        }

        Ok(Self { client: Client::with_config(openai_config), model: config.model })
    }

    /// Create a client for an OpenAI-compatible API.
    pub fn compatible(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self, AdkError> {
        let config = OpenAIConfig::compatible(api_key.into(), base_url.into(), model.into());
        Self::new(config)
    }
}

#[async_trait]
impl Llm for OpenAIClient {
    fn name(&self) -> &str {
        &self.model
    }

    async fn generate_content(
        &self,
        request: LlmRequest,
        _stream: bool, // OpenAI always uses streaming internally
    ) -> Result<adk_core::LlmResponseStream, AdkError> {
        let model = self.model.clone();
        let client = self.client.clone();

        let stream = try_stream! {
            // Convert messages
            let messages: Vec<_> = request.contents.iter().map(convert::content_to_message).collect();

            // Build request
            let mut request_builder = CreateChatCompletionRequestArgs::default();
            request_builder.model(&model).messages(messages);

            // Add tools if present
            if !request.tools.is_empty() {
                let tools = convert::convert_tools(&request.tools);
                request_builder.tools(tools);
            }

            // Add generation config
            if let Some(config) = &request.config {
                if let Some(temp) = config.temperature {
                    request_builder.temperature(temp);
                }
                if let Some(top_p) = config.top_p {
                    request_builder.top_p(top_p);
                }
                if let Some(max_tokens) = config.max_output_tokens {
                    request_builder.max_tokens(max_tokens as u32);
                }
                // Note: top_k is not supported by OpenAI API
            }

            let openai_request = request_builder.build()
                .map_err(|e| AdkError::Model(format!("Failed to build request: {}", e)))?;

            // Make streaming request
            let mut stream = client
                .chat()
                .create_stream(openai_request)
                .await
                .map_err(|e| AdkError::Model(format!("OpenAI API error: {}", e)))?;

            // Process stream chunks
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        let response = convert::from_openai_chunk(&chunk);
                        yield response;
                    }
                    Err(e) => {
                        yield Err(AdkError::Model(format!("Stream error: {}", e)))?;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

/// Azure OpenAI client.
pub struct AzureOpenAIClient {
    client: Client<AsyncAzureConfig>,
    deployment_id: String,
}

impl AzureOpenAIClient {
    /// Create a new Azure OpenAI client.
    pub fn new(config: AzureConfig) -> Result<Self, AdkError> {
        let azure_config = AsyncAzureConfig::new()
            .with_api_base(&config.api_base)
            .with_api_version(&config.api_version)
            .with_deployment_id(&config.deployment_id)
            .with_api_key(&config.api_key);

        Ok(Self { client: Client::with_config(azure_config), deployment_id: config.deployment_id })
    }
}

#[async_trait]
impl Llm for AzureOpenAIClient {
    fn name(&self) -> &str {
        &self.deployment_id
    }

    async fn generate_content(
        &self,
        request: LlmRequest,
        _stream: bool, // Azure OpenAI always uses streaming internally
    ) -> Result<adk_core::LlmResponseStream, AdkError> {
        let deployment_id = self.deployment_id.clone();
        let client = self.client.clone();

        let stream = try_stream! {
            // Convert messages
            let messages: Vec<_> = request.contents.iter().map(convert::content_to_message).collect();

            // Build request (Azure uses deployment_id as model)
            let mut request_builder = CreateChatCompletionRequestArgs::default();
            request_builder.model(&deployment_id).messages(messages);

            // Add tools if present
            if !request.tools.is_empty() {
                let tools = convert::convert_tools(&request.tools);
                request_builder.tools(tools);
            }

            // Add generation config
            if let Some(config) = &request.config {
                if let Some(temp) = config.temperature {
                    request_builder.temperature(temp);
                }
                if let Some(top_p) = config.top_p {
                    request_builder.top_p(top_p);
                }
                if let Some(max_tokens) = config.max_output_tokens {
                    request_builder.max_tokens(max_tokens as u32);
                }
            }

            let openai_request = request_builder.build()
                .map_err(|e| AdkError::Model(format!("Failed to build request: {}", e)))?;

            // Make streaming request
            let mut stream = client
                .chat()
                .create_stream(openai_request)
                .await
                .map_err(|e| AdkError::Model(format!("Azure OpenAI API error: {}", e)))?;

            // Process stream chunks
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        let response = convert::from_openai_chunk(&chunk);
                        yield response;
                    }
                    Err(e) => {
                        yield Err(AdkError::Model(format!("Stream error: {}", e)))?;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}
