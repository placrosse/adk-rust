# RMCP 0.14 Migration Analysis

## Overview

This document analyzes our MCP implementation in `adk-tool` against rmcp 0.14's capabilities and breaking changes from 0.13.

## Current Implementation Status

### What We Use (adk-tool/src/mcp/toolset.rs)

| Feature | Status | Notes |
|---------|--------|-------|
| `RoleClient` | ✅ Used | Client-side MCP integration |
| `CallToolRequestParams` | ✅ Updated | Renamed from `CallToolRequestParam`, added `meta: None` |
| `RawContent` handling | ✅ Used | Text, Image, Resource, Audio, ResourceLink |
| `RunningService` | ✅ Used | Service lifecycle management |
| `list_all_tools()` | ✅ Used | Tool discovery |
| `call_tool()` | ✅ Used | Tool execution |
| `cancellation_token()` | ✅ Used | Graceful shutdown |

### Features We Enable

```toml
# Base client features
rmcp = { version = "0.14", features = ["client"] }

# With HTTP transport (optional)
rmcp = { version = "0.14", features = ["client", "transport-streamable-http-client-reqwest"] }
```

## Breaking Changes from 0.13 to 0.14

### 1. `CallToolRequestParam` → `CallToolRequestParams` ✅ Fixed

The type was renamed (note the 's' at the end):

```rust
// Before (0.13)
use rmcp::model::CallToolRequestParam;

// After (0.14)
use rmcp::model::CallToolRequestParams;
```

### 2. New `meta` Field Required ✅ Fixed

```rust
// Before (0.13)
CallToolRequestParam {
    name: ...,
    arguments: ...,
    task: None,
}

// After (0.14)
CallToolRequestParams {
    name: ...,
    arguments: ...,
    task: None,
    meta: None,  // New required field
}
```

### 3. HTTP Transport API Changes ✅ Fixed

The streamable HTTP client transport API changed:

```rust
// Before (0.13) - builder pattern
StreamableHttpClientTransport::builder(&endpoint)
    .timeout(timeout)
    .header(key, value)
    .build()?

// After (0.14) - from_uri pattern
StreamableHttpClientTransport::from_uri(endpoint.as_str())
// or
StreamableHttpClientTransport::from_config(config)
```

## New Features in 0.14

### 1. Task Support (SEP-1686) ✅ Implemented

RMCP 0.14 fully supports the task lifecycle for long-running operations:

- **Create**: Set `task` field on `CallToolRequestParams` to enqueue async operations
- **Inspect**: Use `tasks/get` to retrieve task status
- **Await**: Use `tasks/result` to block until completion
- **Cancel**: Use `tasks/cancel` to terminate running tasks

**ADK Implementation**: We've added `McpTaskConfig` and task polling in `McpTool::execute()`.

### 2. Streamable HTTP Transport ✅ Implemented

For remote MCP servers, use the streamable HTTP transport:

```rust
use adk_tool::McpHttpClientBuilder;

let toolset = McpHttpClientBuilder::new("https://mcp.example.com/v1")
    .with_auth(McpAuth::bearer("token"))
    .timeout(Duration::from_secs(60))
    .connect()
    .await?;
```

### 3. OAuth2 Authentication ✅ Implemented

We've added `McpAuth` and `OAuth2Config` for authenticated MCP connections:

```rust
use adk_tool::{McpAuth, OAuth2Config};

let auth = McpAuth::oauth2(
    OAuth2Config::new("client-id", "https://auth.example.com/token")
        .with_secret("client-secret")
        .with_scopes(vec!["mcp:read".into()])
);
```

## Implementation Summary

### New Files Added

| File | Purpose |
|------|---------|
| `adk-tool/src/mcp/task.rs` | Task configuration and status types |
| `adk-tool/src/mcp/auth.rs` | Authentication types (Bearer, API Key, OAuth2) |
| `adk-tool/src/mcp/http.rs` | HTTP transport builder |

### Updated Files

| File | Changes |
|------|---------|
| `adk-tool/Cargo.toml` | Added `http-transport` feature, upgraded to rmcp 0.14 |
| `adk-tool/src/mcp/toolset.rs` | Task polling logic, updated to `CallToolRequestParams` |
| `adk-tool/src/mcp/mod.rs` | Export new types |
| `adk-tool/src/lib.rs` | Export `McpHttpClientBuilder` |
| `examples/Cargo.toml` | Upgraded to rmcp 0.14 |
| `docs/.../mcp_test/Cargo.toml` | Upgraded to rmcp 0.14 |

### Feature Flags

```toml
[features]
default = []
# Enable HTTP transport for remote MCP servers
http-transport = ["rmcp/transport-streamable-http-client-reqwest", "reqwest"]
```

## API Examples

### Task Support

```rust
use adk_tool::{McpToolset, McpTaskConfig};
use std::time::Duration;

let toolset = McpToolset::new(client)
    .with_task_support(
        McpTaskConfig::enabled()
            .poll_interval(Duration::from_secs(2))
            .timeout(Duration::from_secs(300))
    );
```

### HTTP Transport with OAuth

```rust
use adk_tool::{McpHttpClientBuilder, McpAuth, OAuth2Config};

let toolset = McpHttpClientBuilder::new("https://mcp.example.com/v1")
    .with_auth(McpAuth::oauth2(
        OAuth2Config::new("client-id", "https://auth.example.com/token")
            .with_secret("client-secret")
    ))
    .connect()
    .await?;
```

### Simple Bearer Auth

```rust
let toolset = McpHttpClientBuilder::new("https://mcp.example.com/v1")
    .with_auth(McpAuth::bearer("my-api-token"))
    .connect()
    .await?;
```

## Conclusion

The upgrade from rmcp 0.13 to 0.14 is complete with:

- ✅ All breaking changes addressed
- ✅ Task support implemented for long-running tools
- ✅ HTTP transport support added (behind feature flag)
- ✅ OAuth2 authentication support added
- ✅ All tests passing
- ✅ All examples compiling

The implementation follows the MCP specification and integrates cleanly with ADK's existing tool infrastructure.
