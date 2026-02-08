# ADK Spatial OS Roadmap

## Reframe

This roadmap replaces the current "3D UI demo" framing with an **AI-native spatial operating shell** where:

- users work through familiar desktop interaction patterns,
- one Master Prompt routes intent,
- apps are ADK-Rust agent runtimes,
- trust and approvals are first-class.

## Product Thesis

People should not learn a new computing language to use agents. They should use normal language in a shell that feels like the next logical step after macOS/Windows style UX.

## What Changes from the Prototype

1. Prototype goal: render scenes.
2. New goal: run an operating shell for agent apps.
3. Prototype center: visuals.
4. New center: intent routing, app orchestration, trust, and continuity.

## Foundational Concepts

- **Master Prompt**: universal intent input.
- **Agent App**: packaged ADK-Rust agent with capabilities, permissions, lifecycle hooks.
- **Spatial Workspace**: practical, layered context model with 2D fallback.
- **Trust Panel**: approvals, risk, and audit in one place.
- **Activity Timeline**: transparent execution and handoffs.

## Phases

1. Contract Reset
2. Shell MVP
3. Agent App Runtime
4. Master Prompt Orchestration
5. Trust + Approval + Audit
6. Spatial Compositor + UX Polish
7. Developer Adoption + Release

## Key Deliverables

- New spec set: `.kiro/specs/adk-spatial-os/`
- New shell crate (planned): `adk-spatial-os`
- App SDK/manifests for ADK-Rust agents
- Contract test suite for shell/app runtime events

## Success Criteria

- Users can run multi-app agent workflows from normal prompts.
- Dangerous actions cannot execute without approval.
- App handoffs are visible and user-controllable.
- Shell remains responsive and usable for mainstream desktop workflows.
