# =============================================================================
# ADK-Rust Development Environment (devenv.nix)
# =============================================================================
# Reproducible dev environment using devenv.sh (https://devenv.sh)
#
# Setup:
#   1. Install devenv: https://devenv.sh/getting-started/
#   2. Run: devenv shell
#   3. Everything is ready — cargo, sccache, mold, cmake, protobuf, node, etc.
#
# This gives identical toolchains on Linux, macOS, and CI.
# =============================================================================

{ pkgs, lib, ... }:

{
  # --------------------------------------------------------------------------
  # Core Rust toolchain
  # --------------------------------------------------------------------------
  languages.rust = {
    enable = true;
    channel = "stable";
    # Pin to the version in Cargo.toml [workspace.package] rust-version
    # Update this when bumping MSRV
  };

  languages.javascript = {
    enable = true;
    package = pkgs.nodejs_22;
  };

  # --------------------------------------------------------------------------
  # System packages available in the dev shell
  # --------------------------------------------------------------------------
  packages = with pkgs; [
    # Build essentials
    cmake              # Required for audiopus (openai-webrtc feature)
    pkg-config
    openssl

    # Fast linker (Linux)
    mold

    # Compilation cache — dramatically speeds up rebuilds and CI
    sccache

    # Protobuf (for gRPC codegen if needed)
    protobuf

    # Frontend tooling (ADK Studio UI)
    nodePackages.pnpm

    # Utilities
    just               # Modern make alternative (optional)
    git
    jq
    curl
  ]
  ++ lib.optionals pkgs.stdenv.isLinux [
    # Linux-only: faster linking, perf tools
    clang
    lld
  ];

  # --------------------------------------------------------------------------
  # Environment variables
  # --------------------------------------------------------------------------
  env = {
    # Enable sccache as Cargo's compiler wrapper
    RUSTC_WRAPPER = "sccache";

    # cmake 4.x compat for audiopus builds
    CMAKE_POLICY_VERSION_MINIMUM = "3.5";

    # Cargo incremental builds
    CARGO_INCREMENTAL = "1";

    # Sparse registry protocol (faster index updates)
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL = "sparse";
  };

  # --------------------------------------------------------------------------
  # Shell hooks — run on entering the dev shell
  # --------------------------------------------------------------------------
  enterShell = ''
    echo "🦀 ADK-Rust dev environment ready"
    echo "   Rust:    $(rustc --version)"
    echo "   Cargo:   $(cargo --version)"
    echo "   sccache: $(sccache --version 2>/dev/null || echo 'not found')"
    echo "   Node:    $(node --version)"
    echo ""
    echo "   Run 'make help' for build commands"
  '';

  # --------------------------------------------------------------------------
  # Pre-commit hooks (optional, enable if desired)
  # --------------------------------------------------------------------------
  # pre-commit.hooks = {
  #   rustfmt.enable = true;
  #   clippy.enable = true;
  # };
}
