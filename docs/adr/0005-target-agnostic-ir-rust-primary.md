# 0005 — Target-agnostic IR; Rust primary through 1.0

- **Status:** Accepted
- **Date:** 2026-07-12
- **Deciders:** Felipe Carvajal Brown

## Context

Early framing treated "generates Rust" as absolute. Revisited: the project shouldn't be painted into a corner architecturally, even though Rust is and remains the entire point of the product through 1.0.

## Decision

The Typed IR is designed target-agnostic from day one: graph → IR → backend, where "backend" is a pluggable concept, not graph → IR → Rust-only. Rust is the only backend actually implemented and shipped through 1.0. No other backend (GDScript, C#, etc.) is built, scoped, or scheduled — this ADR only prevents the IR layer from hard-coding Rust-specific assumptions that would make a future backend a rewrite instead of an addition.

## Consequences

- The IR layer needs a small amount of extra design discipline up front (don't leak Rust AST types into the IR itself) but no extra implementation work — there is exactly one backend (Rust AST Generator) through 1.0.
- Does not change MVP or 1.0 scope in [ADR-0002](0002-mvp-scope-cli-scripting-only.md) or the roadmap.
- Future backend work, if ever pursued, gets its own ADR when it's actually scheduled — this ADR is not a commitment to build one.

## Alternatives considered

- **Rust-only IR, no target-agnostic discipline.** Simpler short-term, but risks Rust AST types leaking throughout the Semantic Analyzer and canvas layers, making any future backend a rewrite rather than an addition. Rejected given the project explicitly wants to stay adaptable.
- **Multi-backend from day one (build GDScript or C# alongside Rust now).** Rejected as scope creep — no user need for it yet, and it would slow MVP delivery for a hypothetical.
