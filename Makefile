# ADK-Rust Makefile
# Common build commands for development

.PHONY: help setup check-env build build-all test test-all clippy fmt clean examples docs

# Default target
help:
	@echo "ADK-Rust Build Commands"
	@echo "======================="
	@echo ""
	@echo "Setup:"
	@echo "  make setup        - Install/check dev tools (sccache, mold, cmake)"
	@echo "  make check-env    - Check what's installed without changing anything"
	@echo ""
	@echo "Basic Commands:"
	@echo "  make build        - Build all workspace crates (default features)"
	@echo "  make build-all    - Build workspace with all features"
	@echo "  make test         - Run all tests"
	@echo "  make clippy       - Run clippy lints"
	@echo "  make fmt          - Format code"
	@echo "  make clean        - Clean build artifacts"
	@echo ""
	@echo "Examples:"
	@echo "  make examples     - Build all examples (CPU-only, no GPU required)"
	@echo "  make examples-gpu - Build examples with Metal GPU support (macOS)"
	@echo ""
	@echo "mistral.rs (Local LLM - excluded from workspace by default):"
	@echo "  make build-mistralrs       - Build adk-mistralrs (CPU-only)"
	@echo "  make build-mistralrs-metal - Build with Metal GPU (macOS)"
	@echo "  make build-mistralrs-cuda  - Build with CUDA GPU (requires toolkit)"
	@echo ""
	@echo "Feature-Specific Builds:"
	@echo "  make build-openai     - Build with OpenAI support"
	@echo "  make build-anthropic  - Build with Anthropic support"
	@echo "  make build-ollama     - Build with Ollama support"
	@echo ""
	@echo "Cache:"
	@echo "  make cache-stats  - Show sccache hit/miss statistics"
	@echo "  make cache-clear  - Clear sccache and cargo caches"
	@echo ""
	@echo "Documentation:"
	@echo "  make docs         - Generate documentation"
	@echo ""
	@echo "Note: adk-mistralrs is excluded from workspace to allow --all-features"
	@echo "      to work without CUDA toolkit. Build it explicitly with:"
	@echo "      make build-mistralrs"

# ---------------------------------------------------------------------------
# Setup & environment
# ---------------------------------------------------------------------------

# Install dev tools (sccache, mold, cmake, etc.)
setup:
	@./scripts/setup-dev.sh

# Check what's installed without changing anything
check-env:
	@./scripts/setup-dev.sh --check

# Show sccache statistics
cache-stats:
	@sccache --show-stats 2>/dev/null || echo "sccache not installed. Run: make setup"

# Clear all caches
cache-clear:
	@sccache --stop-server 2>/dev/null || true
	cargo clean
	@echo "Caches cleared."

# ---------------------------------------------------------------------------
# Build commands
# ---------------------------------------------------------------------------

# Build all workspace crates (CPU-only, safe for all systems)
build:
	cargo build --workspace

# Build workspace with all features (safe - adk-mistralrs excluded)
build-all:
	cargo build --workspace --all-features

# Build with release optimizations
build-release:
	cargo build --workspace --release

# Run all tests
test:
	cargo test --workspace

# Run clippy lints
clippy:
	cargo clippy --workspace

# Format code
fmt:
	cargo fmt --all

# Clean build artifacts
clean:
	cargo clean

# Build examples (CPU-only, works on all systems)
examples:
	cargo build --examples --features "openai,anthropic,deepseek,ollama,groq,browser,guardrails,sso"

# Build examples with mistralrs (CPU-only)
examples-mistralrs:
	cargo build --examples --features "openai,anthropic,deepseek,ollama,groq,mistralrs,browser,guardrails,sso"

# Build examples with Metal GPU support (macOS only)
examples-gpu:
	cargo build --examples --features "openai,anthropic,deepseek,ollama,groq,mistralrs,browser,guardrails,sso,metal"

# Feature-specific builds
build-openai:
	cargo build --workspace --features "openai"

build-anthropic:
	cargo build --workspace --features "anthropic"

build-ollama:
	cargo build --workspace --features "ollama"

# mistral.rs builds (excluded from workspace, must build explicitly)
build-mistralrs:
	cargo build --manifest-path adk-mistralrs/Cargo.toml

build-mistralrs-metal:
	cargo build --manifest-path adk-mistralrs/Cargo.toml --features "metal"

build-mistralrs-cuda:
	@echo "Note: Requires NVIDIA CUDA toolkit installed"
	cargo build --manifest-path adk-mistralrs/Cargo.toml --features "cuda"

# Generate documentation
docs:
	cargo doc --workspace --no-deps --open

# Run a specific mistralrs example
run-mistralrs-basic:
	cargo run --example mistralrs_basic --features mistralrs

run-mistralrs-basic-metal:
	cargo run --example mistralrs_basic --features mistralrs,metal
