# adk-mistralrs

Native [mistral.rs](https://github.com/EricLBuehler/mistral.rs) integration for ADK-Rust, providing blazingly fast local LLM inference without external dependencies like Ollama.

> **Note:** This crate is NOT published to crates.io because mistral.rs depends on unpublished git dependencies. Add it via git dependency instead.

## Features

- **Native Rust Integration**: Direct embedding of mistral.rs, no daemon required
- **ISQ (In-Situ Quantization)**: Quantize models on-the-fly at load time
- **PagedAttention**: Memory-efficient attention for longer contexts
- **Multi-Device Support**: CPU, CUDA, Metal acceleration
- **Multimodal**: Vision, speech, and diffusion model support
- **LoRA/X-LoRA**: Adapter support with hot-swapping
- **Tool Calling**: Full function calling support via ADK interface

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
adk-core = "0.1"
adk-agent = "0.1"

# mistral.rs support (git dependency - not on crates.io)
adk-mistralrs = { git = "https://github.com/zavora-ai/adk-rust" }
```

### With Hardware Acceleration

```toml
# macOS with Metal
adk-mistralrs = { git = "https://github.com/zavora-ai/adk-rust", features = ["metal"] }

# NVIDIA GPU with CUDA
adk-mistralrs = { git = "https://github.com/zavora-ai/adk-rust", features = ["cuda"] }

# CUDA with Flash Attention
adk-mistralrs = { git = "https://github.com/zavora-ai/adk-rust", features = ["flash-attn"] }

# Intel MKL
adk-mistralrs = { git = "https://github.com/zavora-ai/adk-rust", features = ["mkl"] }

# Apple Accelerate
adk-mistralrs = { git = "https://github.com/zavora-ai/adk-rust", features = ["accelerate"] }
```

## Quick Start

```rust
use adk_mistralrs::{MistralRsModel, MistralRsConfig, ModelSource};
use adk_core::Llm;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load a model from HuggingFace
    let model = MistralRsModel::from_hf("microsoft/Phi-3.5-mini-instruct").await?;
    
    // Or load a GGUF file
    // let model = MistralRsModel::from_gguf("path/to/model.gguf").await?;
    
    // Use with ADK agents
    let request = LlmRequest::new("Hello, how are you?");
    let response = model.generate_content(request, false).await?;
    
    println!("{}", response);
    Ok(())
}
```

### With ISQ Quantization

```rust
use adk_mistralrs::{MistralRsModel, MistralRsConfig, ModelSource, QuantizationLevel};

let config = MistralRsConfig::builder()
    .model_source(ModelSource::HuggingFace("mistralai/Mistral-7B-v0.1".into()))
    .isq(QuantizationLevel::Q4_0)
    .build();

let model = MistralRsModel::new(config).await?;
```

### With Tool Calling

```rust
use adk_mistralrs::MistralRsModel;
use adk_core::{Llm, LlmRequest};
use serde_json::json;

let model = MistralRsModel::from_hf("mistralai/Mistral-7B-Instruct-v0.3").await?;

let tools = json!({
    "get_weather": {
        "description": "Get current weather for a location",
        "parameters": {
            "type": "object",
            "properties": {
                "location": { "type": "string" }
            },
            "required": ["location"]
        }
    }
});

let request = LlmRequest::new("What's the weather in Tokyo?")
    .with_tools(tools);

let response = model.generate_content(request, false).await?;
```

## Supported Models

- **Text**: Mistral, Llama, Phi, Qwen, Gemma, and more
- **Vision**: LLaVa, Qwen-VL, Gemma 3
- **Speech**: Dia 1.6b
- **Diffusion**: FLUX.1
- **Embedding**: EmbeddingGemma, Qwen3 Embedding

## Why Not crates.io?

mistral.rs depends on the `candle` ML framework from HuggingFace, which uses git dependencies for its crates. crates.io doesn't allow publishing crates with git dependencies, so this crate must be added via git.

This is a common pattern for ML crates in Rust that depend on rapidly-evolving frameworks.

## License

MIT License - see [LICENSE](../LICENSE)
