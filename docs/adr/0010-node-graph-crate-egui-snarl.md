# 0010 — Node-graph canvas crate: egui_snarl

- **Status:** Accepted.
- **Date:** 2026-07-24
- **Deciders:** Felipe Carvajal Brown

## Context

[ADR-0009](0009-editor-host-standalone-egui-app.md) settled the editor host as a
standalone `egui`/`eframe` app and committed to using a maintained `egui`
node-graph crate for the canvas rather than hand-rolling ports, wires, zoom, and
pan. It named `egui_snarl` and `egui_node_graph2` as the leading candidates but
deliberately deferred the specific pick to "a current maintenance/version check"
during the editor's implementation brainstorm. This ADR records that check and
makes the pick.

The check was run on 2026-07-24 against crates.io and docs.rs:

| Crate | Latest version | Released | egui requirement |
|-------|----------------|----------|------------------|
| `egui` (baseline) | 0.35.0 | 2026-06-25 | — |
| `egui_snarl` | 0.11.0 | 2026-06-29 | `^0.35` (current) |
| `egui_node_graph2` | 0.7.0 | 2024-11-03 | `^0.29` (six releases behind) |

`egui_snarl` shipped a release tracking the current `egui` (0.35) four days after
`egui` 0.35 itself, and is actively maintained. `egui_node_graph2` had not
released since November 2024 and pins `egui ^0.29`; adopting it would drag the
whole editor back to an `egui` version six releases old, which conflicts with the
project's commitment to `egui` for both the editor and the eventual GUI-output
backend ([ADR-0006](0006-output-gui-toolkit-egui-swappable.md)) and would force a
later forced migration.

## Decision

Build the `vr-editor` canvas on **`egui_snarl`** (0.11.x, tracking `egui` 0.35).
`vr-editor` renders the graph onto an `egui_snarl` `Snarl` via a `SnarlViewer`
implementation and reads its node/wire state; per
[ADR-0009](0009-editor-host-standalone-egui-app.md), that state is adapted into a
`vr_graph::Graph` for `validate()` and `lower()`.

This ADR fixes the widget crate only. It does not change the toolkit family
(`egui`, fixed by ADR-0009) or any backend decision.

## Consequences

- The editor tracks the current `egui` line, keeping the editor and the future
  GUI-output backend ([ADR-0006](0006-output-gui-toolkit-egui-swappable.md)) on
  one `egui` version rather than straddling 0.29 and 0.35.
- Canvas primitives (ports, wires, zoom, pan, node bodies) come from a maintained
  crate, as ADR-0009 required; the editor's job is the `Graph <-> Snarl` mapping,
  not drawing a graph widget from scratch.
- The editor gains a dependency on `egui_snarl`'s release cadence for future
  `egui` bumps. Given its demonstrated four-day turnaround on `egui` 0.35, this is
  a low but real coupling; if it ever lags, the give-back policy in
  [ADR-0011](0011-upstream-improvements-to-dependencies.md) applies (contribute
  the bump upstream rather than fork).
- If `egui_snarl` is later found to be missing something the editor needs
  (e.g. a specific interaction the MVP requires), the first response is an
  upstream contribution per [ADR-0011](0011-upstream-improvements-to-dependencies.md),
  not a private fork.

## Alternatives considered

- **`egui_node_graph2`** — rejected: last released 2024-11-03 and pinned to
  `egui ^0.29`, six releases behind current `egui`. Adopting it would pin the
  editor to a stale `egui` and force a later migration, against the ADR-0006
  `egui` commitment.
- **Hand-roll the canvas on bare `egui`** — rejected by
  [ADR-0009](0009-editor-host-standalone-egui-app.md) already: it discards the
  real ports/wires/zoom/pan work a maintained crate donates. Reserved only for the
  case where no maintained crate fits, which the check shows is not the situation.
