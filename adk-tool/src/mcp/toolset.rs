// MCP (Model Context Protocol) Toolset Integration
//
// Based on Go implementation: adk-go/tool/mcptoolset/
// Uses official Rust SDK: https://github.com/modelcontextprotocol/rust-sdk
//
// The McpToolset connects to an MCP server, discovers available tools,
// and exposes them as ADK-compatible tools for use with LlmAgent.

use super::task::{McpTaskConfig, TaskError, TaskStatus};
use adk_core::{AdkError, ReadonlyContext, Result, Tool, ToolContext, Toolset};
use async_trait::async_trait;
use rmcp::{
    RoleClient,
    model::{CallToolRequestParams, RawContent, ResourceContents},
    service::RunningService,
};
use serde_json::{Value, json};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// Type alias for tool filter predicate
pub type ToolFilter = Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// Sanitize JSON schema for LLM compatibility.
/// Removes fields like `$schema`, `additionalProperties`, `definitions`, `$ref`
/// that some LLM APIs (like Gemini) don't accept.
fn sanitize_schema(value: &mut Value) {
    if let Value::Object(map) = value {
        map.remove("$schema");
        map.remove("definitions");
        map.remove("$ref");
        map.remove("additionalProperties");

        for (_, v) in map.iter_mut() {
            sanitize_schema(v);
        }
    } else if let Value::Array(arr) = value {
        for v in arr.iter_mut() {
            sanitize_schema(v);
        }
    }
}

/// MCP Toolset - connects to an MCP server and exposes its tools as ADK tools.
///
/// This toolset implements the ADK `Toolset` trait and bridges the gap between
/// MCP servers and ADK agents. It:
/// 1. Connects to an MCP server via the provided transport
/// 2. Discovers available tools from the server
/// 3. Converts MCP tools to ADK-compatible `Tool` implementations
/// 4. Proxies tool execution calls to the MCP server
///
/// # Example
///
/// ```rust,ignore
/// use adk_tool::McpToolset;
/// use rmcp::{ServiceExt, transport::TokioChildProcess};
/// use tokio::process::Command;
///
/// // Create MCP client connection to a local server
/// let client = ().serve(TokioChildProcess::new(
///     Command::new("npx")
///         .arg("-y")
///         .arg("@modelcontextprotocol/server-everything")
/// )?).await?;
///
/// // Create toolset from the client
/// let toolset = McpToolset::new(client);
///
/// // Add to agent
/// let agent = LlmAgentBuilder::new("assistant")
///     .toolset(Arc::new(toolset))
///     .build()?;
/// ```
pub struct McpToolset<S = ()>
where
    S: rmcp::service::Service<RoleClient> + Send + Sync + 'static,
{
    /// The running MCP client service
    client: Arc<Mutex<RunningService<RoleClient, S>>>,
    /// Optional filter to select which tools to expose
    tool_filter: Option<ToolFilter>,
    /// Name of this toolset
    name: String,
    /// Task configuration for long-running operations
    task_config: McpTaskConfig,
}

impl<S> McpToolset<S>
where
    S: rmcp::service::Service<RoleClient> + Send + Sync + 'static,
{
    /// Create a new MCP toolset from a running MCP client service.
    ///
    /// The client should already be connected and initialized.
    /// Use `rmcp::ServiceExt::serve()` to create the client.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use rmcp::{ServiceExt, transport::TokioChildProcess};
    /// use tokio::process::Command;
    ///
    /// let client = ().serve(TokioChildProcess::new(
    ///     Command::new("my-mcp-server")
    /// )?).await?;
    ///
    /// let toolset = McpToolset::new(client);
    /// ```
    pub fn new(client: RunningService<RoleClient, S>) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
            tool_filter: None,
            name: "mcp_toolset".to_string(),
            task_config: McpTaskConfig::default(),
        }
    }

    /// Set a custom name for this toolset.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Enable task support for long-running operations.
    ///
    /// When enabled, tools marked as `is_long_running()` will use MCP's
    /// async task lifecycle (SEP-1686) instead of blocking calls.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let toolset = McpToolset::new(client)
    ///     .with_task_support(McpTaskConfig::enabled()
    ///         .poll_interval(Duration::from_secs(2))
    ///         .timeout(Duration::from_secs(300)));
    /// ```
    pub fn with_task_support(mut self, config: McpTaskConfig) -> Self {
        self.task_config = config;
        self
    }

    /// Add a filter to select which tools to expose.
    ///
    /// The filter function receives a tool name and returns true if the tool
    /// should be included.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let toolset = McpToolset::new(client)
    ///     .with_filter(|name| {
    ///         matches!(name, "read_file" | "list_directory" | "search_files")
    ///     });
    /// ```
    pub fn with_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.tool_filter = Some(Arc::new(filter));
        self
    }

    /// Add a filter that only includes tools with the specified names.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let toolset = McpToolset::new(client)
    ///     .with_tools(&["read_file", "write_file"]);
    /// ```
    pub fn with_tools(self, tool_names: &[&str]) -> Self {
        let names: Vec<String> = tool_names.iter().map(|s| s.to_string()).collect();
        self.with_filter(move |name| names.iter().any(|n| n == name))
    }

    /// Get a cancellation token that can be used to shutdown the MCP server.
    ///
    /// Call `cancel()` on the returned token to cleanly shutdown the MCP server.
    /// This should be called before exiting to avoid EPIPE errors.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let toolset = McpToolset::new(client);
    /// let cancel_token = toolset.cancellation_token().await;
    ///
    /// // ... use the toolset ...
    ///
    /// // Before exiting:
    /// cancel_token.cancel();
    /// ```
    pub async fn cancellation_token(&self) -> rmcp::service::RunningServiceCancellationToken {
        let client = self.client.lock().await;
        client.cancellation_token()
    }
}

#[async_trait]
impl<S> Toolset for McpToolset<S>
where
    S: rmcp::service::Service<RoleClient> + Send + Sync + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn tools(&self, _ctx: Arc<dyn ReadonlyContext>) -> Result<Vec<Arc<dyn Tool>>> {
        let client = self.client.lock().await;

        // List all tools from the MCP server (handles pagination internally)
        let mcp_tools = client
            .list_all_tools()
            .await
            .map_err(|e| AdkError::Tool(format!("Failed to list MCP tools: {}", e)))?;

        // Convert MCP tools to ADK tools
        let mut tools: Vec<Arc<dyn Tool>> = Vec::new();

        for mcp_tool in mcp_tools {
            let tool_name = mcp_tool.name.to_string();

            // Apply filter if present
            if let Some(ref filter) = self.tool_filter {
                if !filter(&tool_name) {
                    continue;
                }
            }

            let adk_tool = McpTool {
                name: tool_name,
                description: mcp_tool.description.map(|d| d.to_string()).unwrap_or_default(),
                input_schema: {
                    let mut schema = Value::Object(mcp_tool.input_schema.as_ref().clone());
                    sanitize_schema(&mut schema);
                    Some(schema)
                },
                output_schema: mcp_tool.output_schema.map(|s| {
                    let mut schema = Value::Object(s.as_ref().clone());
                    sanitize_schema(&mut schema);
                    schema
                }),
                client: self.client.clone(),
                is_long_running: false, // TODO: detect from MCP tool annotations
                task_config: self.task_config.clone(),
            };

            tools.push(Arc::new(adk_tool) as Arc<dyn Tool>);
        }

        Ok(tools)
    }
}

/// Individual MCP tool wrapper that implements the ADK `Tool` trait.
///
/// This struct wraps an MCP tool and proxies execution calls to the MCP server.
struct McpTool<S>
where
    S: rmcp::service::Service<RoleClient> + Send + Sync + 'static,
{
    name: String,
    description: String,
    input_schema: Option<Value>,
    output_schema: Option<Value>,
    client: Arc<Mutex<RunningService<RoleClient, S>>>,
    /// Whether this tool is long-running (from MCP tool metadata)
    is_long_running: bool,
    /// Task configuration
    task_config: McpTaskConfig,
}

impl<S> McpTool<S>
where
    S: rmcp::service::Service<RoleClient> + Send + Sync + 'static,
{
    /// Poll a task until completion or timeout
    async fn poll_task(
        &self,
        client: &RunningService<RoleClient, S>,
        task_id: &str,
    ) -> std::result::Result<Value, TaskError> {
        let start = Instant::now();
        let mut attempts = 0u32;

        loop {
            // Check timeout
            if let Some(timeout_ms) = self.task_config.timeout_ms {
                let elapsed = start.elapsed().as_millis() as u64;
                if elapsed >= timeout_ms {
                    return Err(TaskError::Timeout {
                        task_id: task_id.to_string(),
                        elapsed_ms: elapsed,
                    });
                }
            }

            // Check max attempts
            if let Some(max_attempts) = self.task_config.max_poll_attempts {
                if attempts >= max_attempts {
                    return Err(TaskError::MaxAttemptsExceeded {
                        task_id: task_id.to_string(),
                        attempts,
                    });
                }
            }

            // Wait before polling
            tokio::time::sleep(self.task_config.poll_duration()).await;
            attempts += 1;

            debug!(
                task_id = task_id,
                attempt = attempts,
                "Polling MCP task status"
            );

            // Poll task status using tasks/get
            // Note: This requires the MCP server to support SEP-1686 task lifecycle
            let poll_result = client
                .call_tool(CallToolRequestParams {
                    name: "tasks/get".into(),
                    arguments: Some(serde_json::Map::from_iter([(
                        "task_id".to_string(),
                        Value::String(task_id.to_string()),
                    )])),
                    task: None,
                    meta: None,
                })
                .await
                .map_err(|e| TaskError::PollFailed(e.to_string()))?;

            // Parse task status from response
            let status = self.parse_task_status(&poll_result)?;

            match status {
                TaskStatus::Completed => {
                    debug!(task_id = task_id, "Task completed successfully");
                    // Extract result from the poll response
                    return self.extract_task_result(&poll_result);
                }
                TaskStatus::Failed => {
                    let error_msg = self.extract_error_message(&poll_result);
                    return Err(TaskError::TaskFailed {
                        task_id: task_id.to_string(),
                        error: error_msg,
                    });
                }
                TaskStatus::Cancelled => {
                    return Err(TaskError::Cancelled(task_id.to_string()));
                }
                TaskStatus::Pending | TaskStatus::Running => {
                    // Continue polling
                    debug!(
                        task_id = task_id,
                        status = ?status,
                        "Task still in progress"
                    );
                }
            }
        }
    }

    /// Parse task status from poll response
    fn parse_task_status(
        &self,
        result: &rmcp::model::CallToolResult,
    ) -> std::result::Result<TaskStatus, TaskError> {
        // Try to extract status from structured content first
        if let Some(ref structured) = result.structured_content {
            if let Some(status_str) = structured.get("status").and_then(|v| v.as_str()) {
                return match status_str {
                    "pending" => Ok(TaskStatus::Pending),
                    "running" => Ok(TaskStatus::Running),
                    "completed" => Ok(TaskStatus::Completed),
                    "failed" => Ok(TaskStatus::Failed),
                    "cancelled" => Ok(TaskStatus::Cancelled),
                    _ => {
                        warn!(status = status_str, "Unknown task status");
                        Ok(TaskStatus::Running) // Assume still running
                    }
                };
            }
        }

        // Try to extract from text content
        for content in &result.content {
            if let Some(text_content) = content.deref().as_text() {
                // Try to parse as JSON
                if let Ok(parsed) = serde_json::from_str::<Value>(&text_content.text) {
                    if let Some(status_str) = parsed.get("status").and_then(|v| v.as_str()) {
                        return match status_str {
                            "pending" => Ok(TaskStatus::Pending),
                            "running" => Ok(TaskStatus::Running),
                            "completed" => Ok(TaskStatus::Completed),
                            "failed" => Ok(TaskStatus::Failed),
                            "cancelled" => Ok(TaskStatus::Cancelled),
                            _ => Ok(TaskStatus::Running),
                        };
                    }
                }
            }
        }

        // Default to running if we can't determine status
        Ok(TaskStatus::Running)
    }

    /// Extract result from completed task
    fn extract_task_result(
        &self,
        result: &rmcp::model::CallToolResult,
    ) -> std::result::Result<Value, TaskError> {
        // Try structured content first
        if let Some(ref structured) = result.structured_content {
            if let Some(output) = structured.get("result") {
                return Ok(json!({ "output": output }));
            }
            return Ok(json!({ "output": structured }));
        }

        // Fall back to text content
        let mut text_parts: Vec<String> = Vec::new();
        for content in &result.content {
            if let Some(text_content) = content.deref().as_text() {
                text_parts.push(text_content.text.clone());
            }
        }

        if text_parts.is_empty() {
            Ok(json!({ "output": null }))
        } else {
            Ok(json!({ "output": text_parts.join("\n") }))
        }
    }

    /// Extract error message from failed task
    fn extract_error_message(&self, result: &rmcp::model::CallToolResult) -> String {
        // Try structured content
        if let Some(ref structured) = result.structured_content {
            if let Some(error) = structured.get("error").and_then(|v| v.as_str()) {
                return error.to_string();
            }
        }

        // Try text content
        for content in &result.content {
            if let Some(text_content) = content.deref().as_text() {
                return text_content.text.clone();
            }
        }

        "Unknown error".to_string()
    }

    /// Extract task ID from create task response
    fn extract_task_id(
        &self,
        result: &rmcp::model::CallToolResult,
    ) -> std::result::Result<String, TaskError> {
        // Try structured content
        if let Some(ref structured) = result.structured_content {
            if let Some(task_id) = structured.get("task_id").and_then(|v| v.as_str()) {
                return Ok(task_id.to_string());
            }
        }

        // Try text content (might be JSON)
        for content in &result.content {
            if let Some(text_content) = content.deref().as_text() {
                if let Ok(parsed) = serde_json::from_str::<Value>(&text_content.text) {
                    if let Some(task_id) = parsed.get("task_id").and_then(|v| v.as_str()) {
                        return Ok(task_id.to_string());
                    }
                }
            }
        }

        Err(TaskError::CreateFailed(
            "No task_id in response".to_string(),
        ))
    }
}

#[async_trait]
impl<S> Tool for McpTool<S>
where
    S: rmcp::service::Service<RoleClient> + Send + Sync + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn is_long_running(&self) -> bool {
        self.is_long_running
    }

    fn parameters_schema(&self) -> Option<Value> {
        self.input_schema.clone()
    }

    fn response_schema(&self) -> Option<Value> {
        self.output_schema.clone()
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let client = self.client.lock().await;

        // Determine if we should use task mode
        let use_task_mode = self.task_config.enable_tasks && self.is_long_running;

        if use_task_mode {
            debug!(
                tool = self.name,
                "Executing tool in task mode (long-running)"
            );

            // Create task request with task parameters
            let task_params = self.task_config.to_task_params();
            let task_map = task_params.as_object().cloned();

            let create_result = client
                .call_tool(CallToolRequestParams {
                    name: self.name.clone().into(),
                    arguments: if args.is_null() || args == json!({}) {
                        None
                    } else {
                        match args {
                            Value::Object(map) => Some(map),
                            _ => {
                                return Err(AdkError::Tool(
                                    "Tool arguments must be an object".to_string(),
                                ));
                            }
                        }
                    },
                    task: task_map,
                    meta: None,
                })
                .await
                .map_err(|e| {
                    AdkError::Tool(format!("Failed to create task for '{}': {}", self.name, e))
                })?;

            // Extract task ID
            let task_id = self.extract_task_id(&create_result).map_err(|e| {
                AdkError::Tool(format!("Failed to get task ID: {}", e))
            })?;

            debug!(tool = self.name, task_id = task_id, "Task created, polling for completion");

            // Poll for completion
            let result = self.poll_task(&client, &task_id).await.map_err(|e| {
                AdkError::Tool(format!("Task execution failed: {}", e))
            })?;

            return Ok(result);
        }

        // Standard synchronous execution
        let result = client
            .call_tool(CallToolRequestParams {
                name: self.name.clone().into(),
                arguments: if args.is_null() || args == json!({}) {
                    None
                } else {
                    // Convert Value to the expected type
                    match args {
                        Value::Object(map) => Some(map),
                        _ => {
                            return Err(AdkError::Tool(
                                "Tool arguments must be an object".to_string(),
                            ));
                        }
                    }
                },
                task: None,
                meta: None,
            })
            .await
            .map_err(|e| {
                AdkError::Tool(format!("Failed to call MCP tool '{}': {}", self.name, e))
            })?;

        // Check for error response
        if result.is_error.unwrap_or(false) {
            let mut error_msg = format!("MCP tool '{}' execution failed", self.name);

            // Extract error details from content
            for content in &result.content {
                // Use Deref to access the inner RawContent
                if let Some(text_content) = content.deref().as_text() {
                    error_msg.push_str(": ");
                    error_msg.push_str(&text_content.text);
                    break;
                }
            }

            return Err(AdkError::Tool(error_msg));
        }

        // Return structured content if available
        if let Some(structured) = result.structured_content {
            return Ok(json!({ "output": structured }));
        }

        // Otherwise, collect text content
        let mut text_parts: Vec<String> = Vec::new();

        for content in &result.content {
            // Access the inner RawContent via Deref
            let raw: &RawContent = content.deref();
            match raw {
                RawContent::Text(text_content) => {
                    text_parts.push(text_content.text.clone());
                }
                RawContent::Image(image_content) => {
                    // Return image data as base64
                    text_parts.push(format!(
                        "[Image: {} bytes, mime: {}]",
                        image_content.data.len(),
                        image_content.mime_type
                    ));
                }
                RawContent::Resource(resource_content) => {
                    let uri = match &resource_content.resource {
                        ResourceContents::TextResourceContents { uri, .. } => uri,
                        ResourceContents::BlobResourceContents { uri, .. } => uri,
                    };
                    text_parts.push(format!("[Resource: {}]", uri));
                }
                RawContent::Audio(_) => {
                    text_parts.push("[Audio content]".to_string());
                }
                RawContent::ResourceLink(link) => {
                    text_parts.push(format!("[ResourceLink: {}]", link.uri));
                }
            }
        }

        if text_parts.is_empty() {
            return Err(AdkError::Tool(format!("MCP tool '{}' returned no content", self.name)));
        }

        Ok(json!({ "output": text_parts.join("\n") }))
    }
}

// Ensure McpTool is Send + Sync
unsafe impl<S> Send for McpTool<S> where
    S: rmcp::service::Service<RoleClient> + Send + Sync + 'static
{
}
unsafe impl<S> Sync for McpTool<S> where
    S: rmcp::service::Service<RoleClient> + Send + Sync + 'static
{
}
