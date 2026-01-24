# MCP Enhanced Features Design

## Overview

This document outlines the implementation of three enhanced MCP capabilities:
1. **Task Support** - Async task lifecycle for long-running tools
2. **Streamable HTTP Transport** - Default transport for remote MCP servers
3. **OAuth Integration** - Authentication for remote MCP servers

## 1. Task Support (SEP-1686)

### Current State

Our `Tool` trait already has `is_long_running()`:

```rust
pub trait Tool: Send + Sync {
    fn is_long_running(&self) -> bool { false }
    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value>;
}
```

### Proposed Changes

#### 1.1 New `McpTaskConfig` struct

```rust
/// Configuration for MCP task-based execution
#[derive(Debug, Clone, Default)]
pub struct McpTaskConfig {
    /// Enable task mode for long-running tools
    pub enable_tasks: bool,
    /// Default poll interval in milliseconds
    pub poll_interval_ms: u64,
    /// Maximum wait time before timeout
    pub timeout_ms: Option<u64>,
}
```

#### 1.2 Enhanced `McpToolset`

```rust
pub struct McpToolset<S> {
    client: Arc<Mutex<RunningService<RoleClient, S>>>,
    tool_filter: Option<ToolFilter>,
    name: String,
    task_config: McpTaskConfig,  // NEW
}

impl<S> McpToolset<S> {
    /// Enable task mode for long-running operations
    pub fn with_task_support(mut self, config: McpTaskConfig) -> Self {
        self.task_config = config;
        self
    }
}
```

#### 1.3 Task-aware Tool Execution

```rust
async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
    let client = self.client.lock().await;
    
    // Determine if we should use task mode
    let use_task = self.task_config.enable_tasks && self.is_long_running;
    
    let result = if use_task {
        // Create task request
        let task_request = CallToolRequestParam {
            name: self.name.clone().into(),
            arguments: args_to_map(args)?,
            task: Some(json!({
                "poll_interval_ms": self.task_config.poll_interval_ms
            }).as_object().cloned().unwrap()),
        };
        
        // Submit task
        let create_result = client.call_tool(task_request).await?;
        let task_id = extract_task_id(&create_result)?;
        
        // Poll for completion
        self.poll_task(&client, &task_id).await?
    } else {
        // Synchronous execution
        client.call_tool(CallToolRequestParam {
            name: self.name.clone().into(),
            arguments: args_to_map(args)?,
            task: None,
        }).await?
    };
    
    convert_result(result)
}
```

## 2. Streamable HTTP Transport

### Current State

We only support stdio transport via `TokioChildProcess`.

### Proposed Changes

#### 2.1 New Feature Flag

```toml
# adk-tool/Cargo.toml
[features]
default = []
http-transport = ["rmcp/transport-streamable-http-client-reqwest"]
```

#### 2.2 New `McpHttpClient` Builder

```rust
/// Builder for HTTP-based MCP connections
pub struct McpHttpClientBuilder {
    endpoint: String,
    auth: Option<McpAuth>,
    timeout: Duration,
    headers: HashMap<String, String>,
}

impl McpHttpClientBuilder {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            auth: None,
            timeout: Duration::from_secs(30),
            headers: HashMap::new(),
        }
    }
    
    /// Add OAuth2 authentication
    pub fn with_oauth(mut self, auth: McpAuth) -> Self {
        self.auth = Some(auth);
        self
    }
    
    /// Add custom header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    
    /// Build and connect
    pub async fn connect(self) -> Result<McpToolset<impl Service<RoleClient>>> {
        let transport = StreamableHttpClientTransport::builder()
            .endpoint(&self.endpoint)
            .timeout(self.timeout)
            .build()?;
        
        let client = ().serve(transport).await?;
        Ok(McpToolset::new(client))
    }
}
```

#### 2.3 Unified Connection API

```rust
/// Connect to an MCP server (auto-detects transport)
pub enum McpConnection {
    /// Local process via stdio
    Process(Command),
    /// Remote server via HTTP
    Http(McpHttpClientBuilder),
}

impl McpConnection {
    pub async fn connect(self) -> Result<McpToolset<DynService>> {
        match self {
            McpConnection::Process(cmd) => {
                let client = ().serve(TokioChildProcess::new(cmd)?).await?;
                Ok(McpToolset::new(client).into_dyn())
            }
            McpConnection::Http(builder) => {
                builder.connect().await.map(|t| t.into_dyn())
            }
        }
    }
}
```

## 3. OAuth Integration

### Current State

`adk-auth` has SSO/OAuth support but not integrated with MCP.

### Proposed Changes

#### 3.1 MCP Auth Types

```rust
/// Authentication for MCP connections
pub enum McpAuth {
    /// No authentication
    None,
    /// Bearer token (static)
    Bearer(String),
    /// OAuth2 with automatic refresh
    OAuth2(OAuth2Config),
    /// Use adk-auth SSO provider
    Sso(Arc<dyn TokenValidator>),
}

#[derive(Debug, Clone)]
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub token_url: String,
    pub scopes: Vec<String>,
    /// Cached token (refreshed automatically)
    token: Arc<RwLock<Option<CachedToken>>>,
}
```

#### 3.2 Integration with adk-auth

```rust
use adk_auth::sso::{TokenValidator, TokenClaims};

impl McpHttpClientBuilder {
    /// Use adk-auth SSO provider for authentication
    pub fn with_sso_provider<V: TokenValidator + 'static>(
        mut self, 
        validator: V
    ) -> Self {
        self.auth = Some(McpAuth::Sso(Arc::new(validator)));
        self
    }
}
```

#### 3.3 Token Refresh Middleware

```rust
/// Middleware that handles token refresh for MCP HTTP transport
struct AuthMiddleware {
    auth: McpAuth,
}

impl AuthMiddleware {
    async fn get_token(&self) -> Result<String> {
        match &self.auth {
            McpAuth::None => Ok(String::new()),
            McpAuth::Bearer(token) => Ok(token.clone()),
            McpAuth::OAuth2(config) => config.get_or_refresh_token().await,
            McpAuth::Sso(validator) => {
                // Get token from SSO provider
                validator.get_access_token().await
            }
        }
    }
}
```

## Implementation Plan

### Phase 1: Task Support (Low Risk)

1. Add `McpTaskConfig` struct
2. Add `with_task_support()` to `McpToolset`
3. Implement task polling in `McpTool::execute()`
4. Add tests for task lifecycle
5. Update examples

### Phase 2: Streamable HTTP (Medium Risk)

1. Add `http-transport` feature flag
2. Implement `McpHttpClientBuilder`
3. Add `McpConnection` enum for unified API
4. Add integration tests with mock HTTP server
5. Update documentation

### Phase 3: OAuth Integration (Medium Risk)

1. Add `McpAuth` enum
2. Implement OAuth2 token refresh
3. Integrate with `adk-auth::sso`
4. Add authentication middleware
5. Add examples for authenticated MCP

## API Examples

### Task Support

```rust
let toolset = McpToolset::new(client)
    .with_task_support(McpTaskConfig {
        enable_tasks: true,
        poll_interval_ms: 1000,
        timeout_ms: Some(60_000),
    });
```

### HTTP Transport

```rust
let toolset = McpHttpClientBuilder::new("https://mcp.example.com/v1")
    .with_oauth(OAuth2Config {
        client_id: "my-app".into(),
        token_url: "https://auth.example.com/token".into(),
        scopes: vec!["mcp:read".into(), "mcp:write".into()],
        ..Default::default()
    })
    .connect()
    .await?;
```

### With adk-auth SSO

```rust
use adk_auth::sso::GoogleProvider;

let provider = GoogleProvider::new("client-id");
let toolset = McpHttpClientBuilder::new("https://mcp.example.com/v1")
    .with_sso_provider(provider)
    .connect()
    .await?;
```

## File Changes Summary

| File | Changes |
|------|---------|
| `adk-tool/Cargo.toml` | Add `http-transport` feature |
| `adk-tool/src/mcp/mod.rs` | Export new types |
| `adk-tool/src/mcp/toolset.rs` | Add task support |
| `adk-tool/src/mcp/http.rs` | NEW: HTTP transport |
| `adk-tool/src/mcp/auth.rs` | NEW: Auth types |
| `adk-tool/src/mcp/task.rs` | NEW: Task polling |
| `examples/mcp_http/` | NEW: HTTP example |
| `examples/mcp_oauth/` | NEW: OAuth example |
