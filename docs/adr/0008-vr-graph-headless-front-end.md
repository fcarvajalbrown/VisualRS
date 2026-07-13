# 0008 — vr-graph: a headless graph front-end producing IR

- **Status:** Accepted
- **Date:** 2026-07-12
- **Deciders:** Felipe Carvajal Brown

## Context

Phase 2 is the editor (Godot `GraphEdit` via `gdext`, per ADR-0003). The
interactive canvas needs the Godot engine installed and a human validating it,
so it cannot be built or tested headlessly. But a large share of Phase 2's value
is pure Rust: the graph data model the canvas manipulates, the lowering from that
model to the Typed IR, and pin type-checking. Those can be built and unit-tested
now, and the eventual plugin can be built on top of them rather than entangled
with them.

## Decision

Introduce `vr-graph`, a crate that defines a Blueprints-style node/pin/wire graph
model and lowers it to a `vr_ir::Program`. It depends on `vr-ir` only: no
`syn`/`quote` (that is `vr-rustgen`), no `godot`/`gdext` (that is the deferred
plugin). Structs and enums are declarative decls; function bodies are graphs of
statement nodes threaded by exec edges plus value nodes wired by typed data edges,
with inline leaves for trivial constants. Lowering feeds the existing
`vr-rustgen` backend unchanged, keeping the pipeline `vr-graph -> vr-ir ->
vr-rustgen`.

The deferred Godot editor plugin will be a separate crate that adapts
`GraphEdit` state into `vr_graph` and calls `validate`/`lower`; it gets its own
ADR when it lands.

## Consequences

- Phase 2's graph-to-Rust seam is provable now (the capstone graph lowers to
  source byte-identical to the hand-authored IR fixture and runs with the same
  output), independent of Godot.
- The ADR-0005 layering is preserved: `vr-graph` is another front-end producing
  the target-agnostic IR; only `vr-rustgen` knows Rust syntax.
- Roadmap Phase 2 stays In Progress until the Godot plugin ships; this decision
  covers only the headless core.
- Full pin-type inference (method/builtin return types) is deferred to Phase 4
  (Standard Library Nodes), which supplies the type table it needs. The headless
  core type-checks only statically-knowable conflicts.

## Alternatives considered

- **Fold the model into the future plugin crate** — rejected: it would couple the
  testable graph/lowering logic to Godot and block headless verification.
- **Skip a graph model and lower Godot `GraphEdit` connections straight to IR in
  the plugin** — rejected: no unit-testable core, and it would reimplement the
  same lowering behind a GUI that only a human can drive.
- **Region/block (AST-adjacent) model without exec/data pins** — rejected during
  brainstorming: it drifts from the `GraphEdit` wire model the real canvas uses,
  so it would need reworking when the plugin lands.
