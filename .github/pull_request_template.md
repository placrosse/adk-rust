## What

Brief description of the change.

## Why

Link to the issue this addresses: Fixes #___

## How

Summary of the approach taken.

## PR Checklist

### Quality Gates (all required)

- [ ] `cargo fmt --all` — code is formatted
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings
- [ ] `cargo test --workspace` — all tests pass
- [ ] Builds clean: `cargo build --workspace`

### Code Quality

- [ ] New code has tests (unit, integration, or property tests as appropriate)
- [ ] Public APIs have rustdoc comments with `# Example` sections
- [ ] No `println!`/`eprintln!` in library code (use `tracing` instead)
- [ ] No hardcoded secrets, API keys, or local paths

### Hygiene

- [ ] No local development artifacts (`.env`, `.DS_Store`, IDE configs, build dirs)
- [ ] No unrelated changes mixed in (formatting, refactoring, other features)
- [ ] Commit messages follow conventional format (`feat:`, `fix:`, `docs:`, etc.)
- [ ] PR targets `main` branch

### Documentation (if applicable)

- [ ] CHANGELOG.md updated for user-facing changes
- [ ] README updated if crate capabilities changed
- [ ] Examples added or updated for new features
