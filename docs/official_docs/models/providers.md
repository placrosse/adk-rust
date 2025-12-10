# Model Providers

ADK-Rust supports multiple LLM providers through the `adk-model` crate. All providers implement the `Llm` trait, making them interchangeable in your agents.

## Supported Providers

| Provider | Models | Feature Flag |
|----------|--------|--------------|
| **Gemini** | gemini-2.5-flash, gemini-2.5-pro, gemini-2.0-flash | (default) |
| **OpenAI** | gpt-4o, gpt-4o-mini, gpt-4-turbo | `openai` |
| **Anthropic** | claude-opus-4, claude-sonnet-4, claude-3.5-sonnet | `anthropic` |
| **DeepSeek** | deepseek-chat, deepseek-reasoner | `deepseek` |

## Installation

```toml
[dependencies]
# All providers
adk-model = { version = "0.1", features = ["all-providers"] }

# Or individual providers
adk-model = { version = "0.1", features = ["openai"] }
adk-model = { version = "0.1", features = ["anthropic"] }
adk-model = { version = "0.1", features = ["deepseek"] }
```

## Environment Variables

```bash
# Google Gemini
export GOOGLE_API_KEY="your-api-key"

# OpenAI
export OPENAI_API_KEY="your-api-key"

# Anthropic
export ANTHROPIC_API_KEY="your-api-key"

# DeepSeek
export DEEPSEEK_API_KEY="your-api-key"
```

## Gemini (Google)

Google's Gemini models are the default provider.

```rust
use adk_model::GeminiModel;
use adk_agent::LlmAgentBuilder;
use std::sync::Arc;

let api_key = std::env::var("GOOGLE_API_KEY")?;
let model = GeminiModel::new(&api_key, "gemini-2.5-flash")?;

let agent = LlmAgentBuilder::new("assistant")
    .model(Arc::new(model))
    .build()?;
```

## OpenAI

```rust
use adk_model::openai::{OpenAIClient, OpenAIConfig};
use adk_agent::LlmAgentBuilder;
use std::sync::Arc;

let api_key = std::env::var("OPENAI_API_KEY")?;
let model = OpenAIClient::new(OpenAIConfig::new(api_key, "gpt-4o"))?;

let agent = LlmAgentBuilder::new("assistant")
    .model(Arc::new(model))
    .build()?;
```

## Anthropic (Claude)

```rust
use adk_model::anthropic::{AnthropicClient, AnthropicConfig};
use adk_agent::LlmAgentBuilder;
use std::sync::Arc;

let api_key = std::env::var("ANTHROPIC_API_KEY")?;
let model = AnthropicClient::new(AnthropicConfig::new(api_key, "claude-sonnet-4-20250514"))?;

let agent = LlmAgentBuilder::new("assistant")
    .model(Arc::new(model))
    .build()?;
```

## DeepSeek

DeepSeek models with unique features like thinking mode and context caching.

```rust
use adk_model::deepseek::{DeepSeekClient, DeepSeekConfig};
use adk_agent::LlmAgentBuilder;
use std::sync::Arc;

let api_key = std::env::var("DEEPSEEK_API_KEY")?;

// Standard chat model
let model = DeepSeekClient::new(DeepSeekConfig::chat(api_key))?;

// Or reasoning model with chain-of-thought
let reasoner = DeepSeekClient::new(DeepSeekConfig::reasoner(api_key))?;

let agent = LlmAgentBuilder::new("assistant")
    .model(Arc::new(model))
    .build()?;
```

### DeepSeek-Specific Features

**Thinking Mode**: The `deepseek-reasoner` model outputs chain-of-thought reasoning:

```rust
let model = DeepSeekClient::new(DeepSeekConfig::reasoner(api_key))?;
// Output includes <thinking>...</thinking> tags with reasoning
```

**Context Caching**: Automatic 10x cost reduction for repeated prefixes (system instructions, documents).

**Tool Calling**: Full function calling support compatible with ADK tools.

## Examples

- `cargo run --example quickstart` - Gemini
- `cargo run --example openai_basic --features openai` - OpenAI
- `cargo run --example anthropic_basic --features anthropic` - Anthropic
- `cargo run --example deepseek_basic --features deepseek` - DeepSeek
- `cargo run --example deepseek_reasoner --features deepseek` - Thinking mode
- `cargo run --example deepseek_tools --features deepseek` - Tool calling

## Related

- [LlmAgent](../agents/llm-agent.md) - Using models with agents
- [Function Tools](../tools/function-tools.md) - Adding tools to agents
