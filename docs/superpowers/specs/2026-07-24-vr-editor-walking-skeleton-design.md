# vr-editor walking skeleton — design

- **Date:** 2026-07-24
- **Status:** Approved (brainstorm), ready for implementation plan
- **Owner:** Felipe Carvajal Brown
- **Related:** [ADR-0009](../../adr/0009-editor-host-standalone-egui-app.md)
  (standalone egui host), [ADR-0010](../../adr/0010-node-graph-crate-egui-snarl.md)
  (egui_snarl canvas), [ADR-0008](../../adr/0008-vr-graph-headless-front-end.md)
  (headless `vr-graph` core), [ADR-0006](../../adr/0006-output-gui-toolkit-egui-swappable.md)
  (egui toolkit)

## 1. Purpose & scope

Stand up `vr-editor`: a standalone `egui`/`eframe` desktop app that opens a
window, renders a small graph on an `egui_snarl` canvas **read-only**, and shows
the live Rust generated from that graph in a side panel. Its single job is to
prove the whole [ADR-0009](../../adr/0009-editor-host-standalone-egui-app.md)
stack renders end-to-end — window, canvas, and the
`vr-graph -> vr-ir -> vr-rustgen` pipeline feeding a live panel — before any
editing UX is built. This is a walking skeleton, deliberately the thinnest honest
proof the stack works, and the base the real editor grows from.

### Out of scope (deferred; YAGNI for the skeleton)

- Adding, deleting, or editing nodes on the canvas (no authoring yet).
- Saving/loading graphs to disk.
- Rendering nested blocks — a `ForEach`/`If`/`Match` node owning its own child
  `Block`. The seed graph is flat precisely so the skeleton does not have to solve
  "a block of nodes living inside a node" yet.
- Exec-wire (statement-order) styling. Only data wires are drawn; statement order
  is conveyed by vertical node placement.
- Pin-level type-error rendering (ROADMAP Phase 2 bullet 3), which lands after
  canvas editing.
- Multiple functions, and visual editing of struct/enum declarations.

## 2. Crate & layering

New crate `crates/editor`, package name `vr-editor`, a binary target.

**Dependencies:** `eframe` + `egui` 0.35, `egui_snarl` 0.11, `vr-graph`,
`vr-rustgen`. It matches its MSRV / edition to the existing workspace crates
(inherited via `workspace = true`).

**Layering (must hold — ADR-0009):** `vr-editor` must **not** depend on
`syn`/`quote`/`proc-macro2` directly (those stay inside `vr-rustgen`), and must
**not** depend on `godot`/`gdext` (post-1.0 output pack only, never the editor).

## 3. Data flow — model is the source of truth (for now)

The seed `vr_graph::Graph` is the authoritative data. A pure `model -> canvas`
function renders it onto `egui_snarl`. The **same** `Graph` is fed to
`validate()` -> `lower()` -> `vr_rustgen::generate()` to produce the panel text.

No canvas -> model adapter exists yet. That adapter (and the "flip" where
`generate`'s input switches from the seed model to the adapter output) arrives
with editing, per ADR-0009's canvas-as-source-of-truth end-state. The
`model -> canvas` renderer written here is permanent — it is also how a saved
graph would later be loaded onto the canvas — so nothing here is throwaway.

## 4. Modules

Small, single-purpose units so the pure logic is testable headlessly and the GUI
shell stays thin.

- **`seed.rs`** — builds the tiny flat seed `Graph` (see §5). Uses
  `vr_graph::build::BlockBuilder` + `vr_graph::model`. No egui.
- **`view.rs`** — **pure, unit-tested.** `to_snarl(&Graph) -> Snarl<NodeView>`:
  maps a function body's nodes to `egui_snarl` nodes and its data edges to wires.
  `NodeView` is the per-node display data (title, input rows, single output).
  Inline literal leaves render as read-only text in the input row. No windowing,
  so it is testable without a display.
- **`codegen.rs`** — **pure, unit-tested.**
  `generate_source(&Graph) -> Result<String, String>`: runs `Graph::validate()`,
  then `vr_graph::lower()` + `vr_rustgen::generate()`, returning the Rust source
  on success or a human-readable, panel-ready error string on failure (validation
  errors joined, or a lowering/codegen error rendered).
- **`app.rs`** — the `eframe::App` shell. Thin, human-validated, **not**
  unit-tested. Holds the `Graph` (source of truth), the derived `Snarl` view, and
  the cached generated source. Implements a read-only `SnarlViewer` that draws
  node titles and pins from `NodeView`; `connect`/`disconnect` are no-ops so the
  canvas cannot be mutated. Layout per §5.
- **`main.rs`** — `eframe::run_native` bootstrap wiring `app.rs`.

## 5. Seed graph & layout

**Seed graph** — a `main()` whose body is, in Rust terms:

```rust
fn main() {
    let n = (1 + 2);
    println!("n: {}", n);
}
```

As `vr_graph` nodes: a `Binary(Add)` node with inline literal leaves `1` and `2`,
wired by a data edge into a `Let { name: "n" }` node; then a
`Builtin(PrintLine("n: {}"))` statement referencing `n`. Statement order
(`Let` then the `println!`) is threaded by `exec` edges with `entry` at the
`Let`. This exercises: value-node rendering, inline-literal rendering, at least
one real data wire, and the full validate/lower/generate path with non-trivial
output.

**Layout:**

```
+------------------------------------------------------+
| Visual Rust   [ File(stub)  Help(stub) ]             |
+---------------------------+--------------------------+
|  canvas (egui_snarl)      |  Generated Rust (read-   |
|                           |  only)                   |
|   (1 + 2) ---> [let n]    |  fn main() {             |
|                           |      let n = (1 + 2);    |
|      [println! "n:{}"]    |      println!("n: {}",n) |
|                           |  }                       |
+---------------------------+--------------------------+
```

- Central panel: the `egui_snarl` canvas.
- Right `SidePanel`: read-only, monospaced generated Rust. If `validate()` fails,
  it shows the error list instead (rendered as error text). The seed graph is
  valid, so it normally shows Rust.
- Top bar: stub menu (`File`, `Help`) with no live actions yet.

## 6. Error handling

`generate_source` never panics: a `validate()` failure yields the joined error
strings, and a `lower()`/`generate()` failure yields that error's `Display`
string. The `app.rs` panel renders whichever string it gets. The GUI thread must
not panic on any graph state.

## 7. Testing & CI

- **Headless unit tests (TDD):**
  - `view::to_snarl` — given the seed `Graph`, assert the produced `Snarl` node
    count, node titles, and data-wire endpoints.
  - `codegen::generate_source` — given the seed `Graph`, assert the output is
    `Ok` and contains `fn main`, `1 + 2`, and `println`.
- **Not auto-tested:** the `eframe`/`winit` shell (`app.rs`, `main.rs`), validated
  by a human running `cargo run -p vr-editor` and eyeballing the window. This is
  ADR-0009's acknowledged windowing constraint — `winit` cannot meaningfully run
  headless in CI without a display.
- **CI:** `vr-editor` joins the workspace and is covered by the existing hard
  gates — `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo build`. No headless GUI run is added to CI.

## 8. Definition of done

- `cargo run -p vr-editor` opens a window showing the seed graph on the canvas and
  the matching generated Rust in the side panel.
- `view` and `codegen` unit tests pass.
- Workspace `fmt` + `clippy -D warnings` + `build` + `test` are green.
- Layering holds: no `syn`/`quote`/`proc-macro2` or `godot`/`gdext` in
  `vr-editor`'s dependency tree.

This closes ROADMAP Phase 2's first bullet (app skeleton) and lays the base for
the canvas-editing, pin-type-check, and live-panel bullets that follow.

## 9. Follow-ups this brainstorm also produced

- [ADR-0010](../../adr/0010-node-graph-crate-egui-snarl.md): node-graph crate =
  `egui_snarl` (records the maintenance/version check ADR-0009 deferred).
- [ADR-0011](../../adr/0011-upstream-improvements-to-dependencies.md): upstream
  improvements to dependencies rather than fork, plus the matching CLAUDE.md rule.
