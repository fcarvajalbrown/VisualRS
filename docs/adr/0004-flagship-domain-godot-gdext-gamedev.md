# 0004 — Flagship post-1.0 domain pack: Godot GDExtension

- **Status:** Accepted
- **Date:** 2026-07-12
- **Deciders:** Felipe Carvajal Brown

## Context

Phase 6 (Plugin SDK) needs a flagship domain to prove the SDK works end-to-end, and the PRD names game development as a target use case. Bevy (pure-Rust ECS engine) was the first candidate researched: strong fit on paper (pure Rust, ECS maps naturally to nodes, active community demand for visual scripting with nothing mature yet built). But [ADR-0003](0003-editor-host-platform-godot-gdext.md) already commits the editor itself to Godot + `gdext`. Choosing Bevy as the flagship domain would mean depending on a *second*, unrelated engine ecosystem purely for the demo domain pack.

Ruled out for the same reason others were ruled out earlier: Flax Engine already ships a mature built-in visual scripting system (redundant to compete with), Fyrox is more monolithic/opinionated (less room to plug into cleanly), GDevelop is TypeScript-based (breaks "Rust is the source of truth").

## Decision

The flagship Phase 6 domain pack targets Godot's own GDExtension API via `gdext` — visual nodes generate real Rust code implementing Godot `Node` classes and game logic, compiled as a native GDExtension and run inside the same engine that hosts the editor.

## Consequences

- Self-consistent story: the tool is built on Godot, and its first real domain pack targets Godot — no second engine dependency to maintain.
- Directly reuses editor-side Godot/`gdext` API knowledge for the codegen side.
- This is explicitly a post-1.0, Phase 6 concern — it does not change MVP or 1.0 core scope (CLI/scripting → async/GUI/Plugin SDK), and Visual Rust's core product remains general-purpose for any Rust developer, not game-dev-locked.

## Alternatives considered

- **Bevy** — pure-Rust ECS, strong fit on paper (active community demand for visual scripting, none mature yet built). Rejected as the *flagship* domain specifically because ADR-0003 already commits the editor to Godot; picking Bevy here would mean maintaining two unrelated engine dependencies instead of one.
- **Flax Engine** — already ships a mature built-in visual scripting system. Rejected: competing head-on with an existing solved problem, not filling a gap.
- **Fyrox** — more monolithic/opinionated (own scene editor, own way of structuring games); less room to plug a visual-Rust domain pack in cleanly.
- **GDevelop** — TypeScript-based engine core. Rejected outright: breaks the "Rust is the source of truth" principle.
