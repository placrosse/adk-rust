# ADK 3D UI Roadmap (`adk-3d-ui`)

## Goal

Build a production-quality **agentic 3D UI runtime** for `adk-rust` where users write normal prompts and an LLM constructs interactive Three.js interfaces via typed scene operations.

Core constraints:
- Keep **v0 simple** (SSE + HTTP, no complex bidirectional protocol).
- Keep output **high-impact visually** (motion, depth, cinematic composition).
- Keep execution **safe and deterministic** (typed ops, approval gates for dangerous actions).

## Product Principles

1. Users should not need 3D language in prompts.
2. Three.js is the renderer; ADK-Rust is the reasoning + control plane.
3. LLM builds from primitives and ops, not fixed templates.
4. Agent actions must be inspectable, reversible, and policy-gated.

## Scope (v0)

### In scope
- New crate: `adk-3d-ui` (Rust server + embedded UI assets).
- Embedded Three.js frontend served by Rust (Studio-like local run).
- Session-scoped SSE stream:
  - `ui_ops`
  - `toast`
  - `done`
  - `error`
  - `ping`
- Client-to-server HTTP events:
  - `select`
  - `command`
  - `approve_action`
- Minimal op set:
  - `create`
  - `patch`
  - `remove`
- Prompt -> Intent -> Scene plan -> `ui_ops` pipeline.
- One end-to-end agentic flow with explicit approval gate.

### Out of scope (v0)
- CRDT/multiplayer collaboration.
- Full websocket protocol + revision conflict resolution.
- Physics simulation, XR, advanced 3D editors.
- Provider-specific rendering hacks.

## Target Demo (v0)

**DevOps “Orbital Ops”**
- 3D service constellation with health pulses.
- Incident/workbench panel on selection.
- Command palette with natural language actions.
- Safe/dangerous action proposals with approve/cancel loop.

## Architecture

### Crate layout (proposed)
- `adk-3d-ui/`
  - `src/`
    - `lib.rs`
    - `server.rs` (axum routes)
    - `session.rs` (session state + channels)
    - `protocol.rs` (ops/events payloads)
    - `planner.rs` (prompt -> intent -> scene plan)
    - `executor.rs` (plan -> ui_ops stream)
    - `policy.rs` (risk tiers + approvals)
  - `ui/` (Three.js + React/Vite app)
  - `assets/` (embedded build artifacts)

### Runtime loop
1. User prompt posted to `run`.
2. Agent plans scene + actions.
3. Server emits `ui_ops` over SSE.
4. Client applies ops in local scene registry.
5. Client sends UI events via HTTP.
6. Agent updates plan/state and streams patches.

## Protocol (v0)

### Server -> Client (SSE)
- `ui_ops`: `{ seq, ops: UiOp[] }`
- `toast`: `{ level, message }`
- `done`: `{ status }`
- `error`: `{ code, message }`
- `ping`: `{ ts }`

### Client -> Server (HTTP)
- `POST /api/3d/event/{session_id}`
  - `select { id }`
  - `command { text }`
  - `approve_action { action_id, approved }`

### Operations
- `create { id, kind, parent?, props }`
- `patch { id, props }`
- `remove { id }`

## Prompt Handling (No 3D Prompt Burden)

### Compiler chain
1. `PromptParser` extracts user intent (goal, entity focus, urgency, tone).
2. `ScenePlanner` maps intent to spatial composition strategy (not template lock-in).
3. `OpGenerator` emits deterministic ops from planner output.
4. `PolicyGate` annotates action ops with risk tiers and approval requirements.

### Examples of normal prompts
- "Show me platform health and what needs attention right now."
- "Why is checkout slow today?"
- "Give me a calm executive summary."
- "Show safest mitigation options for this incident."

## Safety Model

### Risk tiers
- `Safe`: read-only navigation/filtering.
- `Controlled`: acknowledge/annotate/create incident.
- `Dangerous`: restart/scale/rollback.

### Guardrails
- Dangerous actions require explicit `approve_action`.
- Agent must provide concise rationale before high-risk action ops.
- All action decisions are logged with timestamp and outcome.

## Implementation Phases

## Phase 0 - Contract & Skeleton (Week 1)
- Create crate + route skeleton.
- Define protocol structs (`UiOp`, `UiEvent`, SSE payloads).
- Embed static frontend assets.

Acceptance:
- Server starts and serves frontend.
- SSE stream connects and emits `ping`.
- Client can POST a test `select` event.

## Phase 1 - Three.js Runtime + Ops Engine (Week 1-2)
- Implement client scene registry.
- Apply `create/patch/remove`.
- Build minimal primitives:
  - `group`
  - `panel3d`
  - `text3d`
  - `orb`
  - `trail`
  - `command_bar`

Acceptance:
- "Hello world" prompt creates a complete animated 3D scene.
- Ops are applied incrementally without full rerender.

## Phase 2 - Agent Planning Loop (Week 2-3)
- Add prompt -> intent -> scene plan pipeline.
- Connect ADK agent/tools to scene op generation.
- Session state memory for selection/focus.

Acceptance:
- Normal prompt (no 3D terms) yields usable scene.
- `select` and `command` events trigger scene updates.

## Phase 3 - DevOps Vertical Slice (Week 3-4)
- Service constellation view.
- Incident/workbench panel.
- Command palette intent handling.
- Live metric/status patching loop.

Acceptance:
- User can navigate from overview -> service detail -> incident context.
- Status changes patch scene in under 300ms median server-side processing.

## Phase 4 - Agentic Action Flow + Approval (Week 4)
- Add action proposal cards and approval controls.
- Implement risk tiers + policy gate.
- Audit log for proposed/executed actions.

Acceptance:
- Dangerous action cannot execute without explicit approval event.
- Approved action updates UI and audit log state.

## Phase 5 - Polish & Reliability (Week 5)
- Visual polish pass (lighting, transitions, typography).
- Retry/timeout handling for stream and commands.
- Performance budgets and quality gates.

Acceptance:
- Stable 60 FPS target on reference laptop for demo scene.
- Stream recovers cleanly from reconnect.
- No stuck "streaming" state in client.

## Phase 6 - Docs, Examples, Release (Week 5-6)
- Publish crate docs and architecture diagram.
- Add runnable example (`examples/3d_ui_ops_center`).
- Add quickstart and troubleshooting guide.

Acceptance:
- New contributor can run demo in <10 minutes.
- CI passes for protocol unit tests and example smoke test.

## Testing Strategy

### Unit tests (Rust)
- Protocol serialization/deserialization.
- Planner intent parsing and fallback behavior.
- Policy gate behavior by risk tier.

### Integration tests
- SSE stream lifecycle (`ui_ops`, `done`, reconnect).
- UI event roundtrip (`select`, `command`, `approve_action`).
- Session isolation across concurrent sessions.

### Frontend tests
- `applyOps` correctness for create/patch/remove.
- Event emission correctness from interactions.

### Manual demo checklist
- Prompt creates meaningful 3D scene.
- Selecting service updates detail panel.
- Command input changes scene.
- Approval flow works as expected.

## Success Metrics

- Time to first rendered scene after prompt: <= 2.0s median (local dev setup).
- UI event to visible patch latency: <= 500ms p95.
- Action approval safety: 100% dangerous actions blocked without explicit approval.
- Demo wow factor: stakeholders can achieve "normal-language to living 3D UI" in one session.

## Key Risks and Mitigations

1. LLM outputs poor or noisy ops.
- Mitigation: typed validators + op sanitization + quality retry prompt.

2. Visual quality feels generic.
- Mitigation: opinionated visual system (materials, motion presets, depth layering).

3. Stream instability or stuck states.
- Mitigation: explicit terminal signals (`done`, `error`) + heartbeat + client timeout handling.

4. Scope explosion into full editor/game engine.
- Mitigation: strict v0 boundaries and phase gates above.

---

This is a roadmap document. APIs and examples are illustrative and will evolve during implementation.
