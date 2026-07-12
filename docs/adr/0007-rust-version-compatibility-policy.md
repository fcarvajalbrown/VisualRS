# 0007 — Rust version/edition compatibility policy

- **Status:** Accepted
- **Date:** 2026-07-12
- **Deciders:** Felipe Carvajal Brown

## Context

Rust ships stable releases roughly every 6 weeks and periodically cuts new editions. A visual tool that generates and compiles real Rust code will eventually break against upstream changes (new lints becoming errors, edition-gated syntax, deprecated stdlib paths) if there's no explicit process for staying current. This is an engineering/process concern, not a user-facing product feature — no in-app update mechanism is being built for this.

## Decision

- CI runs generated-code validity checks against Rust **stable**, and smoke-tests against **beta** and **nightly** to catch breaking changes before they land in stable.
- An explicit MSRV (Minimum Supported Rust Version) is pinned per minor release of Visual Rust itself and documented in the repo root.
- A new Rust **edition** triggers a dedicated compatibility-review ADR before adoption — not an automatic upgrade.
- New stable Rust syntax/stdlib features are *not* auto-added as nodes. They go into a node-library backlog and get prioritized like any other feature work.

## Consequences

- Requires CI infrastructure against three Rust channels from Phase 1 onward (already reflected in the roadmap's Phase 1 checklist).
- Keeps edition adoption a deliberate decision, not scope creep by default.
- No new product surface, no new maintenance burden beyond CI and this review process.

## Alternatives considered

- **In-app auto-update feature** (editor checks for new Rust releases and pulls updated node definitions/codegen templates). Rejected for now — no node-definition ecosystem exists yet to auto-update, and it implies its own versioning/distribution mechanism. Revisit as a real roadmap phase once the Plugin SDK ([ADR-0004](0004-flagship-domain-godot-gdext-gamedev.md)) has produced a real ecosystem worth auto-updating.
- **No explicit policy — react to breakage as it happens.** Rejected: a 6-week stable cadence means reactive-only handling would produce a steady trickle of surprise breakage rather than caught-in-CI early warning.
