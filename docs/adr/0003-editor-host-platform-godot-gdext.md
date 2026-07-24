# 0003 — Editor host platform: Godot + gdext

- **Status:** Superseded by [0009](0009-editor-host-standalone-egui-app.md)
- **Date:** 2026-07-12
- **Deciders:** Felipe Carvajal Brown

## Context

The original plan was a hand-built canvas on bare `egui`/`egui_node_graph2`. Reconsidering: Godot ships `GraphEdit`/`GraphNode` as built-in engine widgets — a production-ready node-graph UI (ports, wires, zoom, snapping) out of the box. `godot-rust`/`gdext` is mature as of 2026: full Rust bindings for Godot 4, used in production for real editor plugins and tools, not just games. Godot's documentation is also unusually strong (Sphinx/ReadTheDocs, ~500k words, explicit quality processes, translated) — a real asset for a project meant to eventually onboard contributors. Godot is already proven as a non-game application shell (Lorien, an infinite-canvas whiteboard app; Material Maker; Pixelorama all ship on it).

One caveat surfaced in research: GDExtension↔GDScript cross-language calls have measured marshalling overhead. This doesn't apply here — the plugin is 100% Rust via `gdext`, with no GDScript in the hot path.

## Decision

Build the Visual Rust editor as a Godot Engine editor plugin, written entirely in Rust via `gdext`. The infinite canvas uses Godot's built-in `GraphEdit`/`GraphNode` widgets instead of a hand-built `egui` canvas.

This is an implementation detail of the editor tool only. It does not change what generated output programs are or how they run — see the PRD's Non-Goals and "Zero Interpreted Overhead" principle. Godot is a build-time dependency of the *editor*, never a runtime dependency of *generated programs*.

## Consequences

- Most of the from-scratch canvas work (Month/Phase 2 in the original draft) is eliminated in favor of wiring up existing Godot widgets.
- The editor inherits Godot's cross-platform export (Windows/Linux/macOS) for free.
- Introduces a real dependency on the Godot engine and `gdext` API stability for the editor's own development — tracked separately from Rust-language compatibility (see [ADR-0007](0007-rust-version-compatibility-policy.md), which covers the *generated code's* Rust version, not the editor's Godot version).
- Sets up [ADR-0004](0004-flagship-domain-godot-gdext-gamedev.md): since the editor already lives inside the Godot/`gdext` ecosystem, a Godot game-dev domain pack becomes a natural extension rather than a second unrelated engine dependency.

## Alternatives considered

- **Bare `egui` + `egui_node_graph2`** — the original plan. Rejected: means hand-building canvas interaction, docs infrastructure, and cross-platform export from scratch instead of inheriting Godot's.
- **`iced` + `iced-node-editor`** — same category of trade-off as bare egui; no meaningful advantage over Godot's built-in `GraphEdit` for this project's needs.
- **Bevy as the editor host itself** (not just a domain pack) — considered and rejected; Bevy has no built-in node-graph editor widget or mature editor-plugin story comparable to Godot's `EditorPlugin`/`GraphEdit`, so it would need the same from-scratch canvas work as bare egui.
