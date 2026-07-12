# 0001 — Borrow-checker violation visualization

- **Status:** Accepted
- **Date:** 2026-07-12
- **Deciders:** Felipe Carvajal Brown

## Context

Beginners need to instantly recognize an ownership violation without reading raw `rustc` output. Unreal Engine Blueprints — this project's primary UX reference — already trains users to read a red/no-entry pin state as "this connection is invalid," so reusing that convention costs nothing in learnability.

## Decision

- Valid immutable borrow: thin blue wire.
- Valid mutable borrow: thick orange wire.
- Borrow conflict: the offending wire flashes red, a no-entry badge appears at the violating pin, and a hover tooltip states the conflict in plain language (e.g. "value already mutably borrowed at Node X").

## Consequences

- The Semantic Analyzer must attach human-readable conflict messages to specific pins, not just surface raw `rustc` diagnostics.
- The canvas renderer needs per-wire error state, not just per-node error state.
- Worth revisiting the visual vocabulary against RustViz (`rustviz/rustviz`) and BorIs (`ChristianSchott/boris`) before final implementation — both are existing ownership-visualization tools with prior art on this exact problem.

## Alternatives considered

- **Node-level red outline/glow instead of wire.** Closer to how compile errors surface on Blueprint nodes themselves, but less precise — a busy graph with many wires into one node loses which specific borrow conflicted.
- **Wire + node pulse combined.** Maximum clarity but more visual complexity; deferred unless dense graphs prove the wire-only signal gets missed in practice.
- **Timeline/ghost trail** showing a faint history of each borrow/move along the execution path. Rejected for v1 as a bigger implementation lift than the problem (a single point-in-time conflict) needs; may revisit for a dedicated ownership-debugging view later.
