# Gemini Google Cloud SDK Foundation (Roadmap)

Status: Draft

## Goal
Make `adk-gemini` SDK-backed for Google Cloud (Vertex AI) without breaking existing ADK usage.
Only implement features that are explicitly available in the official Rust SDK.

## Sources (SDK Ground Truth)
- `google-cloud-aiplatform-v1` crate docs (streaming RPCs may be missing):
  - https://docs.rs/google-cloud-aiplatform-v1/1.5.0/google_cloud_aiplatform_v1/
- `PredictionService` client methods (`generate_content`, `embed_content`):
  - https://docs.rs/google-cloud-aiplatform-v1/1.5.0/src/google_cloud_aiplatform_v1/client.rs.html
- `google-cloud-auth` credentials modules (API key, service account, external account/ADC):
  - https://docs.rs/google-cloud-auth/1.4.0/google_cloud_auth/credentials/index.html

## Current SDK Capability (Verified)
- `PredictionService::generate_content` is available.
- `PredictionService::embed_content` is available.
- Streaming RPCs are not guaranteed to be exposed in Rust (SDK warning).
- `google-cloud-auth` exposes credentials builders for API key, service account, and external account (WIF/ADC flows).

## Capability Map (SDK vs ADK)

| ADK Feature | SDK Surface (google-cloud-aiplatform-v1) | SDK Support | Notes |
| --- | --- | --- | --- |
| Generate content (non-streaming) | `PredictionService::generate_content` | Yes | Implemented in PR #27. |
| Generate content (streaming) | Streaming RPCs are often missing in Rust SDK | No (per SDK warning) | Do not implement until SDK exposes it. |
| Embed content (single) | `PredictionService::embed_content` | Yes | Implemented in PR #27. |
| Embed content (batch) | No SDK method found | Unknown | Wait for SDK surface. |
| Batch generate | No SDK method found | Unknown | Wait for SDK surface. |
| Files API | No SDK method found | Unknown | Wait for SDK surface. |
| Cache API | No SDK method found | Unknown | Wait for SDK surface. |
| Realtime/Live (bidi) | Streaming RPCs often missing | No (per SDK warning) | Wait for SDK support. |
| Auth: API Key | `google-cloud-auth` API key credentials | Yes | Implemented in PR #27. |
| Auth: Service Account JSON | `google-cloud-auth` service_account builder | Yes | Implemented in PR #27. |
| Auth: WIF / External Account | `google-cloud-auth` external_account builder | Yes | Implemented in PR #27. |

## Implementation Plan (SDK-Only)

### Phase 0 — Verification
- Confirm SDK version used by `adk-gemini` and validate supported methods in that exact version.
- Update this roadmap only when SDK surfaces are verified in docs or source.

### Phase 1 — Core Vertex Support (Done)
- SDK-backed `generate_content` and `embed_content`.
- Credential flows via `google-cloud-auth`.

### Phase 2 — Streaming (Blocked)
- Implement only if SDK exposes streaming RPCs for prediction.
- Until then, return explicit unsupported errors for Vertex backend.

### Phase 3 — Batch, Files, Cache (Blocked)
- Implement only when SDK surfaces appear.
- Do not assume parity with REST endpoints.

### Phase 4 — Default Shift
- Flip to SDK-only for Vertex when parity is sufficient.
- Keep REST backend for non-Vertex (AI Studio) users.
