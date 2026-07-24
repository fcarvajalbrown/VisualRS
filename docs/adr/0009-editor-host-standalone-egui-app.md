# 0009 — Editor host: standalone native app (egui)

- **Status:** Accepted. Supersedes [0003](0003-editor-host-platform-godot-gdext.md).
- **Date:** 2026-07-23
- **Deciders:** Felipe Carvajal Brown

## Context

[ADR-0003](0003-editor-host-platform-godot-gdext.md) made the Visual Rust editor
a Godot editor plugin (`gdext`), reusing Godot's built-in `GraphEdit`/`GraphNode`
widgets for the canvas. In practice that couples the whole tool to a heavy
prerequisite: to run the editor at all, a user (or CI, or a contributor) must
have Godot 4 installed and the plugin loaded inside the Godot editor. For a tool
whose entire pitch is "produce a native, self-contained Rust program," shipping
the authoring environment as a plugin-inside-another-app is an awkward
distribution and onboarding story, and it blocks headless CI of the editor
itself.

Two facts make a different host cheap now:

- [ADR-0008](0008-vr-graph-headless-front-end.md) already split the graph model,
  lowering, and validation into `vr-graph`, a headless crate that depends on
  `vr-ir` only and knows nothing about any GUI. Any editor shell adapts its own
  connection state into a `vr_graph::Graph`, then calls `validate()` and
  `lower()`. The choice of canvas toolkit is therefore isolated to the shell.
- [ADR-0006](0006-output-gui-toolkit-egui-swappable.md) already commits the
  project to `egui` for generated GUI output. Using `egui` for the editor too
  keeps the toolkit surface small: one immediate-mode GUI family across the
  editor and the eventual output backend.

`egui` was in fact the project's *original* editor plan (bare
`egui`/`egui_node_graph2`), recorded in ADR-0003's own "Alternatives considered"
before Godot displaced it. This ADR returns to that direction, but as a
first-class standalone application rather than a hand-built afterthought, now
that mature `egui` node-graph crates exist to supply the canvas.

## Decision

Build the Visual Rust editor as a **standalone native desktop application** written
in Rust with **`egui`** (via `eframe`), producing a self-contained native binary
per OS — a real `.exe` on Windows, plus macOS and Linux builds — with no engine
or runtime to install. The editor lives in a new crate, `vr-editor`.

The node canvas uses an `egui` node-graph crate rather than a hand-rolled one.
The specific crate (leading candidates: `egui_snarl` and `egui_node_graph2`) is
selected during the editor's own implementation brainstorm after a current
maintenance/version check; this ADR fixes the toolkit family (`egui`), not the
individual widget crate.

`vr-editor` adapts the canvas's connection state into a `vr_graph::Graph`, calls
`Graph::validate()` to drive per-pin error rendering, and calls `lower()` plus
`vr-rustgen` to feed a live, read-only "Generated Rust" panel. It depends on
`egui`/`eframe` + the node-graph crate + `vr-graph` + `vr-rustgen`. It does not
depend on `syn`/`quote`/`proc-macro2` directly (that stays inside `vr-rustgen`),
and nothing in the MVP depends on `godot`/`gdext`.

Godot is not retired from the project: it survives only as the post-1.0 game-dev
**output** target ([ADR-0004](0004-flagship-domain-godot-gdext-gamedev.md)),
where graphs generate Godot GDExtension Rust code. Godot is no longer any part of
the editor host.

## Consequences

- The editor ships as a normal cross-platform desktop app with no external engine
  prerequisite, and the editor itself becomes headlessly buildable/testable in CI
  (subject to the usual windowing/`winit` constraints), which the Godot-plugin
  host could not be.
- We give up Godot's free `GraphEdit`/`GraphNode` widget and rebuild the canvas on
  an `egui` node-graph crate. This is the main cost of the pivot: real canvas work
  Godot would have donated. It is bounded by picking a maintained node-graph crate
  rather than hand-building ports/wires/zoom from zero.
- `egui` now spans both the editor and the eventual GUI-output backend
  ([ADR-0006](0006-output-gui-toolkit-egui-swappable.md)), reducing the number of
  distinct UI toolkits the project maintains and de-risking Phase 8.
- The `vr-graph` headless core ([ADR-0008](0008-vr-graph-headless-front-end.md))
  and the whole `vr-graph -> vr-ir -> vr-rustgen` pipeline are unchanged; only the
  shell that consumes `vr-graph` changes. No backend rework.
- The editor's dependency-stability risk shifts from Godot/`gdext` to
  `egui`/`eframe` + the node-graph crate. This is still separate from the
  generated code's Rust-version policy ([ADR-0007](0007-rust-version-compatibility-policy.md)).
- ADR-0004's premise weakens slightly: it argued a Godot output pack is "a natural
  extension" *because the editor already lives in the Godot ecosystem*. That
  rationale no longer holds; the Godot output pack now stands on its own merits as
  a post-1.0 domain pack, not on editor-host adjacency. ADR-0004 stays Accepted
  and Phase 6; only its "already in the ecosystem" justification is void.

## Alternatives considered

- **Keep Godot as the editor host (status quo, ADR-0003)** — rejected: forces a
  Godot install to run the tool, blocks headless editor CI, and makes the
  distribution story a plugin-inside-an-engine rather than a standalone binary.
- **Slint** — Rust-native declarative UI, single binary. Rejected: no ready-made
  node-graph widget (canvas hand-built anyway) and it adds a toolkit the project
  does not otherwise use, unlike `egui` which ADR-0006 already commits to.
- **Tauri (native shell + web frontend)** — rich JS node-graph libraries exist,
  but it splits the stack into Rust + web and ships a heavier app, against the
  all-Rust, single-native-binary goal.
- **Keep an optional Godot-plugin editor front-end as a later phase in addition to
  the standalone app** — rejected for now: two editor front-ends on one headless
  core is maintenance surface with no MVP payoff. Revisit post-1.0 only if
  demand appears; `vr-graph` staying headless keeps the door open.
