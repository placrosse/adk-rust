# RMCP 0.13 Migration Analysis

## Overview

This document analyzes our current MCP implementation in `adk-tool` against rmcp 0.13's new capabilities and breaking changes.

## Current Implementation Status

### What We Use (adk-tool/src/mcp/toolset.rs)

| Feature | Status | Notes |
|---------|--------|-------|
| `RoleClient` | ✅ Used | Client-side MCP integration |
| `CallToolRequestParam` | ✅ Updated | Added `task: None` field for 0.13 |
| `RawContent` handling | ✅ Used | Text, Image, Resource, Audio, ResourceLink |
| `RunningService` | ✅ Used | Service lifecycle management |
| `list_all_tools()` | ✅ Used | Tool discovery |
| `call_tool()` | ✅ Used | Tool execution |
| `cancellation_token()` | ✅ Used | Graceful shutdown |

### Features We Enable

```toml
rmcp = { version = "0.13", features = ["client"] }
```

Only the `client` feature - we're a client consuming MCP servers, not building servers.

## RMCP 0.13 New Capabilities

### 1. Task Support (SEP-1686) ⚠️ Not Yet Utilized

RMCP 0.13 adds task lifecycle for long-running operations:

```rust
// Current implementation - synchronous tool calls
CallToolRequestParam {
    name: "tool_name".into(),
    arguments: Some(args),
    task: None,  // We set this to None
}
```

**Opportunity**: For long-running tools, we could:
- Set `task: Some(task_config)` to enqueue async operations
- Poll with `tasks/get` for status
- Retrieve results with `tasks/result`
- Cancel with `tasks/cancel`

**Impact on ADK**: Our `Tool::is_long_running()` trait method could map to MCP tasks.

### 2. Transport Changes - SSE Deprecated ⚠️ Important

| Transport | Status | Our Usage |
|-----------|--------|-----------|
| `stdio` | ✅ Supported | Primary - via `TokioChildProcess` |
| `SSE` | ❌ Deprecated | Not used |
| `Streamable HTTP` | ✅ New | Not yet used |

**Current**: We use `transport-child-process` (stdio) which is still fully supported.

**Future**: For remote MCP servers, we should use `transport-streamable-http-client` instead of SSE.

### 3. Structured Output with Json Wrapper

RMCP 0.13 has better structured output support:

```rust
// Server-side (if we build MCP servers)
#[tool(name = "calculate")]
async fn calculate(&self, params: Parameters<Request>) -> Result<Json<Response>, String>
```

**Impact**: Our `McpTool` already handles `structured_content` in responses.

### 4. New Content Types

We already handle all content types in 0.13:
- `RawContent::Text` ✅
- `RawContent::Image` ✅
- `RawContent::Resource` ✅
- `RawContent::Audio` ✅
- `RawContent::ResourceLink` ✅

### 5. OAuth Support

RMCP 0.13 adds OAuth2 authentication via the `auth` feature.

**Not currently needed** for our use case (local MCP servers via stdio).

## Breaking Changes Analysis

### 1. `CallToolRequestParam.task` Field ✅ Fixed

```rust
// Before (0.9)
CallToolRequestParam {
    name: ...,
    arguments: ...,
}

// After (0.13)
CallToolRequestParam {
    name: ...,
    arguments: ...,
    task: None,  // New required field
}
```

**Status**: Already fixed in our codebase.

### 2. Transport Module Reorganization

The transport modules have been reorganized:
- `transport::TokioChildProcess` - Still available ✅
- `transport::stdio` - For server-side stdio
- `transport::streamable_http_client` - New HTTP transport
- `transport::streamable_http_server` - New HTTP server transport

**Impact**: Our examples use `TokioChildProcess` which is unchanged.

### 3. Service API Changes

The `ServiceExt` trait and `serve()` method are unchanged for our use case.

## Recommendations

### Short-term (Current Release)

1. ✅ **Done**: Update to rmcp 0.13 with `task: None`
2. ✅ **Done**: Verify all examples compile and work
3. 📝 **Document**: Note that SSE is deprecated in docs

### Medium-term (Next Release)

1. **Add Task Support**: Implement async task handling for long-running tools
   ```rust
   pub struct McpToolset {
       // ... existing fields
       enable_tasks: bool,  // New option
   }
   ```

2. **Add Streamable HTTP Transport**: For remote MCP servers
   ```rust
   // New feature flag
   rmcp = { version = "0.13", features = ["client", "transport-streamable-http-client"] }
   ```

3. **Update Examples**: Add example for remote MCP server via HTTP

### Long-term

1. **MCP Server Support**: Consider adding `adk-mcp-server` crate to expose ADK agents as MCP servers
2. **OAuth Integration**: For authenticated MCP servers

## Feature Flag Recommendations

Current:
```toml
rmcp = { version = "0.13", features = ["client"] }
```

Recommended for full capability:
```toml
rmcp = { version = "0.13", features = [
    "client",
    "transport-child-process",      # stdio (already implied)
    "transport-streamable-http-client",  # For remote servers
] }
```

## Code Changes Summary

| File | Change | Status |
|------|--------|--------|
| `adk-tool/Cargo.toml` | Version 0.9 → 0.13 | ✅ Done |
| `adk-tool/src/mcp/toolset.rs` | Add `task: None` | ✅ Done |
| `examples/Cargo.toml` | Version 0.9 → 0.13 | ✅ Done |
| `docs/.../mcp_test/Cargo.toml` | Version 0.9 → 0.13 | ✅ Done |

## Conclusion

Our MCP integration is **well-aligned** with rmcp 0.13:

- ✅ We use stdio transport (not deprecated SSE)
- ✅ We handle all content types
- ✅ Breaking change (task field) is addressed
- ⚠️ Task support is available but not utilized (future enhancement)
- ⚠️ Streamable HTTP not yet supported (future enhancement for remote servers)

The upgrade to 0.13 is **low risk** and **backward compatible** for our current use cases.
