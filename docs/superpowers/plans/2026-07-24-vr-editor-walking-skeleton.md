# vr-editor Walking Skeleton — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up `vr-editor`, a standalone egui/eframe desktop app that renders a tiny flat seed graph (`let n = 1 + 2; println!("n: {}", n);`) read-only on an `egui_snarl` canvas with a live "Generated Rust" side panel, proving the `vr-graph -> vr-ir -> vr-rustgen` stack renders end-to-end.

**Architecture:** New crate `crates/editor` (package `vr-editor`, a lib + bin). The seed `vr_graph::Graph` is the single source of truth. A pure `to_snarl` renders it onto `egui_snarl`; the same `Graph` feeds `validate() -> lower() -> generate()` to produce the panel text. Pure modules (`seed`, `codegen`, `view`) get headless conformance tests; the eframe shell (`app`, `main`) is human-validated on screen. No canvas -> model adapter exists yet (arrives with editing, per ADR-0009).

**Tech Stack:** Rust (edition 2021, MSRV 1.94), `eframe` + `egui` 0.35, `egui_snarl` 0.11, `vr-graph`, `vr-rustgen`, `vr-ir`.

## Context

This closes ROADMAP Phase 2's first bullet (editor app skeleton). ADR-0009 pivoted the editor host from a Godot plugin to a standalone egui/eframe app; ADR-0010 picked `egui_snarl` 0.11 as the canvas crate. The `vr-graph` headless core and the whole lowering/codegen pipeline already exist and are unchanged — this plan only adds the shell that consumes them. It is a deliberately thin "walking skeleton": the thinnest honest proof the stack renders a window + canvas + live panel before any editing UX is built. The `model -> canvas` renderer written here is permanent (it is also how a saved graph would later load onto the canvas), so nothing here is throwaway. Source spec: `docs/superpowers/specs/2026-07-24-vr-editor-walking-skeleton-design.md`.

## Global Constraints

Every task's requirements implicitly include these. Values copied verbatim from the spec, ADRs, and CLAUDE.md.

- **Edition `2021`, MSRV `rust-version = "1.94"`** — inherit via `.workspace = true`; do not hardcode.
- **Dependencies allowed:** `eframe` + `egui` `0.35`, `egui_snarl` `0.11`, `vr-graph`, `vr-rustgen`, `vr-ir`. Nothing else.
- **Layering (ADR-0009, hard gate):** `vr-editor` must **not** depend on `syn`/`quote`/`proc-macro2` directly, and must **not** depend on `godot`/`gdext`.
- **Workflow is Documentation-Driven (ADR-0012), not TDD-first-by-dogma:** the spec is the documented behavior; pure-logic modules (`seed`, `codegen`, `view`) get mandatory conformance tests; the GUI shell (`app`, `main`) is human-validated on screen, not unit-tested (ADR-0009 windowing constraint — `winit` can't run headless in CI).
- **egui_snarl 0.11 API is version-sensitive.** context7 serves the crate's `main`-branch API, which differs from 0.11. Any `SnarlViewer` / `Snarl::show` / `PinInfo` / `Snarl::nodes`/`wires` signature in this plan is a reference; **confirm each against the installed 0.11 source** (`cargo doc -p egui_snarl --no-deps --open`, or read `~/.cargo/registry/src/*/egui_snarl-0.11.0/`) and adjust — the compiler and the stated assertions are the gate.
- **Commits:** Conventional Commits, committed locally to `main` as work progresses. **Never `git push`** without Felipe's explicit permission in the same turn. **No AI attribution** anywhere (no `Co-Authored-By`, no "Generated with" lines). **No emojis.**

## File Structure

All paths under `C:\Projects\VisualRS`.

- `Cargo.toml` (root) — add `crates/editor` to members; add `egui`/`eframe`/`egui_snarl` to `[workspace.dependencies]`.
- `crates/editor/Cargo.toml` — the new manifest (lib + bin).
- `crates/editor/src/lib.rs` — module declarations (`seed`, `codegen`, `view`, `app`).
- `crates/editor/src/seed.rs` — **pure, tested.** Builds the seed `Graph`.
- `crates/editor/src/codegen.rs` — **pure, tested.** `generate_source(&Graph) -> Result<String, String>`.
- `crates/editor/src/view.rs` — **pure, tested.** `NodeView`, `InputRow`, `to_snarl(&Graph) -> Snarl<NodeView>`.
- `crates/editor/src/app.rs` — **human-validated.** `EditorApp` (`eframe::App`) + read-only `SkeletonViewer`.
- `crates/editor/src/main.rs` — **human-validated.** `eframe::run_native` bootstrap.
- `docs/superpowers/plans/2026-07-24-vr-editor-walking-skeleton.md` — this plan, committed into the repo (DDD convention).

Reference API (verbatim from source, confirmed this session):
- `vr_graph::build::{BlockBuilder, binary, builtin, int, var}` — `BlockBuilder::new()`, `.stmt(NodeKind) -> NodeId`, `.feed(NodeId, u16, Src)`, `.build() -> Block`. `binary(&mut b, BinaryOp, Src, Src) -> Src`, `int(i128) -> Src`, `var(&str) -> Src`, `builtin(&mut b, BuiltinOp, Vec<Src>) -> Src`.
- `vr_graph::{Graph, GraphItem, FunctionGraph, NodeKind}`; `Graph { items: Vec<GraphItem> }`, `FunctionGraph { name, params, ret, body }`, `Block { nodes: BTreeMap<NodeId, Node>, exec, data: Vec<DataEdge>, entry, tail }`, `Node { kind: NodeKind, inline: Vec<(u16, Leaf)> }`, `DataEdge { from, to, to_port: u16 }`, `Leaf::{Lit(Literal), Var(String), Path(Vec<String>)}`.
- `Graph::validate(&self) -> Result<(), Vec<String>>` (method).
- `vr_graph::lower(&Graph) -> Result<vr_ir::Program, vr_graph::LowerError>` (`LowerError: Display`).
- `vr_rustgen::generate(&vr_ir::Program) -> Result<String, vr_rustgen::GenError>` (`GenError: Display`).
- `vr_ir::{BinaryOp::Add, BuiltinOp::PrintLine(String), Literal::{Int(i128),Float(f64),Bool(bool),Char(char),Str(String),Unit}, Type::Unit}`.
- `egui_snarl::{Snarl, InPinId, OutPinId}` — `Snarl::new()`, `insert_node(egui::Pos2, T) -> NodeId`, `connect(OutPinId, InPinId)`, `OutPinId { node, output: usize }`, `InPinId { node, input: usize }`.

---

### Task 1: Crate scaffold wired into the workspace

**Files:**
- Modify: `C:\Projects\VisualRS\Cargo.toml`
- Create: `C:\Projects\VisualRS\crates\editor\Cargo.toml`
- Create: `C:\Projects\VisualRS\crates\editor\src\lib.rs`
- Create: `C:\Projects\VisualRS\crates\editor\src\main.rs`
- Create: `C:\Projects\VisualRS\docs\superpowers\plans\2026-07-24-vr-editor-walking-skeleton.md` (copy of this plan)

**Interfaces:**
- Produces: the `vr-editor` package with a `vr_editor` lib and a `vr-editor` bin, buildable but with no logic yet. Later tasks add modules to `lib.rs`.

- [ ] **Step 1: Add the crate to the workspace and declare GUI deps**

In `C:\Projects\VisualRS\Cargo.toml`, extend the members list and the workspace dependency table:

```toml
[workspace]
resolver = "2"
members = ["crates/ir", "crates/rustgen", "crates/cli", "crates/graph", "crates/editor"]
```

Add these three lines to the existing `[workspace.dependencies]` table (leave the current `syn`/`quote`/`proc-macro2`/`prettyplease`/`vr-*` entries untouched):

```toml
egui = "0.35"
eframe = "0.35"
egui_snarl = "0.11"
```

- [ ] **Step 2: Write the crate manifest**

Create `C:\Projects\VisualRS\crates\editor\Cargo.toml`:

```toml
[package]
name = "vr-editor"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
description = "Visual Rust: standalone egui node-graph editor"

[lib]
name = "vr_editor"

[[bin]]
name = "vr-editor"
path = "src/main.rs"

[dependencies]
vr-ir.workspace = true
vr-graph.workspace = true
vr-rustgen.workspace = true
egui.workspace = true
eframe.workspace = true
egui_snarl.workspace = true
```

- [ ] **Step 3: Write a minimal lib and bin that build**

Create `C:\Projects\VisualRS\crates\editor\src\lib.rs`:

```rust
//! Visual Rust editor (walking skeleton): a standalone egui/eframe app that
//! renders the seed graph read-only on an egui_snarl canvas and shows the live
//! generated Rust. See
//! `docs/superpowers/specs/2026-07-24-vr-editor-walking-skeleton-design.md`.
```

Create `C:\Projects\VisualRS\crates\editor\src\main.rs`:

```rust
//! Binary entry point: boots the eframe window hosting the editor.

fn main() {
    // Wired to the eframe app in Task 5.
}
```

- [ ] **Step 4: Copy this plan into the repo**

Copy the plan file to `C:\Projects\VisualRS\docs\superpowers\plans\2026-07-24-vr-editor-walking-skeleton.md` (DDD convention — the plan is a tracked artifact).

- [ ] **Step 5: Verify the workspace builds**

Run: `cargo build -p vr-editor`
Expected: PASS — the crate compiles (empty lib, empty `main`), and `egui`/`eframe`/`egui_snarl` resolve at 0.35/0.35/0.11.

- [ ] **Step 6: Confirm the layering gate holds early**

Run: `cargo tree -p vr-editor -i syn` then `cargo tree -p vr-editor -i proc-macro2`
Expected: `syn`/`proc-macro2` appear ONLY under `vr-rustgen` (a transitive dep), never as a direct dependency of `vr-editor`. Run `cargo tree -p vr-editor -i gdext 2>&1` — expected: "package ID specification ... did not match any packages" (i.e. absent).

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock crates/editor docs/superpowers/plans
git commit -m "feat(editor): scaffold vr-editor crate wired into the workspace"
```

---

### Task 2: Seed graph builder (`seed.rs`)

**Files:**
- Create: `C:\Projects\VisualRS\crates\editor\src\seed.rs`
- Modify: `C:\Projects\VisualRS\crates\editor\src\lib.rs`

**Interfaces:**
- Consumes: `vr_graph::build::{BlockBuilder, binary, builtin, int, var}`, `vr_graph::{Graph, GraphItem, FunctionGraph, NodeKind}`, `vr_ir::{BinaryOp, BuiltinOp, Type}`.
- Produces: `pub fn seed_graph() -> vr_graph::Graph` — the flat `main()` with 4 body nodes (Binary(Add), Let, Builtin(PrintLine), ExprStmt), 2 data edges. Used by `codegen` and `view` tests.

- [ ] **Step 1: Write the failing test**

Add to `C:\Projects\VisualRS\crates\editor\src\seed.rs` (module body written in Step 3; add the test now so the file exists with the test):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_graph_is_valid() {
        assert!(seed_graph().validate().is_ok());
    }
}
```

Declare the module: add `pub mod seed;` to `C:\Projects\VisualRS\crates\editor\src\lib.rs`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-editor seed`
Expected: FAIL — `cannot find function seed_graph in this scope` (compile error).

- [ ] **Step 3: Write the seed builder**

Prepend to `C:\Projects\VisualRS\crates\editor\src\seed.rs` (above the test module):

```rust
//! The tiny flat seed graph the walking skeleton renders:
//! `let n = 1 + 2; println!("n: {}", n);`. Mirrors vr-graph's own
//! `arithmetic_graph` fixture so it exercises the real builder API.

use vr_graph::build::{binary, builtin, int, var, BlockBuilder};
use vr_graph::{FunctionGraph, Graph, GraphItem, NodeKind};
use vr_ir::{BinaryOp, BuiltinOp, Type};

/// Build the seed `main()` graph:
///
/// ```ignore
/// fn main() {
///     let n = (1 + 2);
///     println!("n: {}", n);
/// }
/// ```
pub fn seed_graph() -> Graph {
    let mut b = BlockBuilder::new();

    // let n = 1 + 2;
    let sum = binary(&mut b, BinaryOp::Add, int(1), int(2));
    let let_n = b.stmt(NodeKind::Let {
        name: "n".into(),
        mutable: false,
    });
    b.feed(let_n, 0, sum);

    // println!("n: {}", n);
    let print = builtin(&mut b, BuiltinOp::PrintLine("n: {}".into()), vec![var("n")]);
    let es = b.stmt(NodeKind::ExprStmt);
    b.feed(es, 0, print);

    Graph {
        items: vec![GraphItem::Function(FunctionGraph {
            name: "main".into(),
            params: vec![],
            ret: Type::Unit,
            body: b.build(),
        })],
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vr-editor seed`
Expected: PASS — `seed_graph_is_valid`.

- [ ] **Step 5: Commit**

```bash
git add crates/editor/src/seed.rs crates/editor/src/lib.rs
git commit -m "feat(editor): add flat seed graph (let n = 1 + 2; println)"
```

---

### Task 3: Live codegen (`codegen.rs`)

**Files:**
- Create: `C:\Projects\VisualRS\crates\editor\src\codegen.rs`
- Modify: `C:\Projects\VisualRS\crates\editor\src\lib.rs`

**Interfaces:**
- Consumes: `seed::seed_graph`, `vr_graph::Graph`, `vr_graph::lower`, `vr_rustgen::generate`.
- Produces: `pub fn generate_source(graph: &vr_graph::Graph) -> Result<String, String>` — runs `validate -> lower -> generate`, never panics; `Ok` is formatted Rust, `Err` is a panel-ready string. Used by `app.rs`.

- [ ] **Step 1: Write the failing test**

Create `C:\Projects\VisualRS\crates\editor\src\codegen.rs` with the test first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::seed_graph;

    #[test]
    fn seed_generates_expected_rust() {
        let src = generate_source(&seed_graph()).expect("seed generates");
        assert!(src.contains("fn main"), "got:\n{src}");
        assert!(src.contains("1 + 2"), "got:\n{src}");
        assert!(src.contains("println"), "got:\n{src}");
    }
}
```

Declare the module: add `pub mod codegen;` to `C:\Projects\VisualRS\crates\editor\src\lib.rs`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-editor codegen`
Expected: FAIL — `cannot find function generate_source in this scope`.

- [ ] **Step 3: Write the implementation**

Prepend to `C:\Projects\VisualRS\crates\editor\src\codegen.rs`:

```rust
//! Pure `Graph -> Rust source` for the live panel. Never panics: any failure is
//! returned as a human-readable string the panel displays verbatim.

use vr_graph::Graph;

/// Run the full `validate -> lower -> generate` pipeline on `graph`, returning
/// formatted Rust on success or a panel-ready error string on failure
/// (validation errors joined by newlines, or a lowering/codegen error's
/// `Display`).
pub fn generate_source(graph: &Graph) -> Result<String, String> {
    graph.validate().map_err(|errs| errs.join("\n"))?;
    let program = vr_graph::lower(graph).map_err(|e| e.to_string())?;
    vr_rustgen::generate(&program).map_err(|e| e.to_string())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vr-editor codegen`
Expected: PASS — `seed_generates_expected_rust`.

- [ ] **Step 5: Commit**

```bash
git add crates/editor/src/codegen.rs crates/editor/src/lib.rs
git commit -m "feat(editor): add generate_source (validate/lower/generate for the live panel)"
```

---

### Task 4: Model -> canvas renderer (`view.rs`)

**Files:**
- Create: `C:\Projects\VisualRS\crates\editor\src\view.rs`
- Modify: `C:\Projects\VisualRS\crates\editor\src\lib.rs`

**Interfaces:**
- Consumes: `seed::seed_graph`, `vr_graph::{Graph, GraphItem}`, `vr_graph::model::{Leaf, Node, NodeId, NodeKind}`, `vr_ir::{BuiltinOp, Literal}`, `egui::pos2`, `egui_snarl::{Snarl, InPinId, OutPinId}`.
- Produces: `pub struct NodeView { pub title: String, pub inputs: Vec<InputRow>, pub has_output: bool }`, `pub enum InputRow { Wired { label: String }, Inline { text: String } }`, `pub fn to_snarl(graph: &vr_graph::Graph) -> egui_snarl::Snarl<NodeView>`. Consumed by `app.rs`'s `SkeletonViewer`.

- [ ] **Step 1: Write the failing tests**

Create `C:\Projects\VisualRS\crates\editor\src\view.rs` with the tests first. NOTE: confirm `Snarl::nodes()` (iterator over `&NodeView`) and `Snarl::wires()` (iterator over wires) exist under those names in egui_snarl 0.11 via `cargo doc -p egui_snarl`; if they differ, keep the assertions (4 nodes, 2 wires, these titles) and adjust only the accessor names.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::seed_graph;

    #[test]
    fn seed_produces_four_nodes() {
        let snarl = to_snarl(&seed_graph());
        assert_eq!(snarl.nodes().count(), 4);
    }

    #[test]
    fn seed_has_two_data_wires() {
        let snarl = to_snarl(&seed_graph());
        assert_eq!(snarl.wires().count(), 2);
    }

    #[test]
    fn titles_cover_the_seed_nodes() {
        let snarl = to_snarl(&seed_graph());
        let titles: Vec<String> = snarl.nodes().map(|n| n.title.clone()).collect();
        assert!(titles.iter().any(|t| t == "let n"), "titles: {titles:?}");
        assert!(titles.iter().any(|t| t == "expr"), "titles: {titles:?}");
        assert!(titles.iter().any(|t| t.contains("Add")), "titles: {titles:?}");
        assert!(titles.iter().any(|t| t.contains("println!")), "titles: {titles:?}");
    }

    #[test]
    fn binary_node_has_two_inline_literal_inputs() {
        let snarl = to_snarl(&seed_graph());
        let add = snarl
            .nodes()
            .find(|n| n.title.contains("Add"))
            .expect("Add node present");
        assert_eq!(add.inputs.len(), 2);
        assert!(matches!(&add.inputs[0], InputRow::Inline { text } if text == "1"));
        assert!(matches!(&add.inputs[1], InputRow::Inline { text } if text == "2"));
        assert!(add.has_output);
    }
}
```

Declare the module: add `pub mod view;` to `C:\Projects\VisualRS\crates\editor\src\lib.rs`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p vr-editor view`
Expected: FAIL — `cannot find function to_snarl` / `cannot find type NodeView`.

- [ ] **Step 3: Write the renderer**

Prepend to `C:\Projects\VisualRS\crates\editor\src\view.rs`:

```rust
//! Pure `model -> egui_snarl` renderer. Testable without a display: it only
//! builds a `Snarl<NodeView>` (node data + positions + wires) and never touches
//! an egui `Ui`. This renderer is permanent — it is also how a saved graph would
//! later be loaded onto the canvas.

use std::collections::BTreeMap;

use egui::pos2;
use egui_snarl::{InPinId, OutPinId, Snarl};

use vr_graph::model::{Leaf, Node, NodeId, NodeKind};
use vr_graph::{Graph, GraphItem};

/// Per-node display data. The `SnarlViewer` in `app.rs` reads this to draw a
/// node's title and pins; this module never touches egui `Ui`.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeView {
    pub title: String,
    pub inputs: Vec<InputRow>,
    pub has_output: bool,
}

/// One input pin's display: either a data-wire target (label only) or an inline
/// literal/variable leaf rendered as read-only text.
#[derive(Clone, Debug, PartialEq)]
pub enum InputRow {
    Wired { label: String },
    Inline { text: String },
}

/// Render the first function body of `graph` onto a fresh `Snarl<NodeView>`:
/// one snarl node per model node (in `NodeId` order, laid out top-to-bottom),
/// one wire per data edge. A graph with no function renders as an empty `Snarl`.
pub fn to_snarl(graph: &Graph) -> Snarl<NodeView> {
    let mut snarl = Snarl::new();

    let Some(body) = graph.items.iter().find_map(|item| match item {
        GraphItem::Function(f) => Some(&f.body),
        _ => None,
    }) else {
        return snarl;
    };

    // Ports that are fed by a data edge, grouped per destination node.
    let mut wired: BTreeMap<NodeId, Vec<u16>> = BTreeMap::new();
    for edge in &body.data {
        wired.entry(edge.to).or_default().push(edge.to_port);
    }

    // Insert one snarl node per model node; remember the id mapping for wiring.
    let mut ids: BTreeMap<NodeId, egui_snarl::NodeId> = BTreeMap::new();
    for (i, (node_id, node)) in body.nodes.iter().enumerate() {
        let ports = wired.get(node_id).map(Vec::as_slice).unwrap_or(&[]);
        let view = build_node_view(node, ports);
        let snarl_id = snarl.insert_node(pos2(0.0, i as f32 * 120.0), view);
        ids.insert(*node_id, snarl_id);
    }

    // One wire per data edge: source output 0 -> destination input `to_port`.
    for edge in &body.data {
        if let (Some(&from), Some(&to)) = (ids.get(&edge.from), ids.get(&edge.to)) {
            snarl.connect(
                OutPinId { node: from, output: 0 },
                InPinId {
                    node: to,
                    input: edge.to_port as usize,
                },
            );
        }
    }

    snarl
}

/// Build the display data for one node. Input rows cover the union of inline
/// ports and wired ports, in ascending port order.
fn build_node_view(node: &Node, wired_ports: &[u16]) -> NodeView {
    let mut ports: Vec<u16> = node
        .inline
        .iter()
        .map(|(p, _)| *p)
        .chain(wired_ports.iter().copied())
        .collect();
    ports.sort_unstable();
    ports.dedup();

    let inputs = ports
        .iter()
        .map(|&p| match node.inline.iter().find(|(ip, _)| *ip == p) {
            Some((_, leaf)) => InputRow::Inline {
                text: leaf_text(leaf),
            },
            None => InputRow::Wired {
                label: port_label(&node.kind, p),
            },
        })
        .collect();

    NodeView {
        title: node_title(&node.kind),
        inputs,
        has_output: is_value_node(&node.kind),
    }
}

/// Human-readable node title. Only the seed's kinds are special-cased; anything
/// else falls back to the kind's `Debug`.
fn node_title(kind: &NodeKind) -> String {
    match kind {
        NodeKind::Let { name, mutable } => {
            if *mutable {
                format!("let mut {name}")
            } else {
                format!("let {name}")
            }
        }
        NodeKind::ExprStmt => "expr".to_string(),
        NodeKind::Binary { op } => format!("{op:?}"),
        NodeKind::Builtin { op } => builtin_title(op),
        other => format!("{other:?}"),
    }
}

fn builtin_title(op: &vr_ir::BuiltinOp) -> String {
    match op {
        vr_ir::BuiltinOp::PrintLine(tmpl) => format!("println!({tmpl:?})"),
        vr_ir::BuiltinOp::EPrintLine(tmpl) => format!("eprintln!({tmpl:?})"),
        other => format!("{other:?}"),
    }
}

/// Whether a node produces a value (has exactly one output pin). Statement /
/// control nodes have none.
fn is_value_node(kind: &NodeKind) -> bool {
    use NodeKind::*;
    matches!(
        kind,
        Field { .. }
            | Call
            | Method { .. }
            | Binary { .. }
            | Ref { .. }
            | StructLit { .. }
            | Builtin { .. }
            | Try
            | Match { .. }
            | If { .. }
            | PathValue(_)
            | VarValue(_)
    )
}

fn port_label(kind: &NodeKind, port: u16) -> String {
    match (kind, port) {
        (NodeKind::Let { .. }, 0) => "value".to_string(),
        (NodeKind::ExprStmt, 0) => "expr".to_string(),
        (NodeKind::Binary { .. }, 0) => "lhs".to_string(),
        (NodeKind::Binary { .. }, 1) => "rhs".to_string(),
        (NodeKind::Builtin { .. }, p) => format!("arg{p}"),
        _ => format!("in{port}"),
    }
}

fn leaf_text(leaf: &Leaf) -> String {
    match leaf {
        Leaf::Lit(lit) => literal_text(lit),
        Leaf::Var(name) => name.clone(),
        Leaf::Path(segs) => segs.join("::"),
    }
}

fn literal_text(lit: &vr_ir::Literal) -> String {
    match lit {
        vr_ir::Literal::Int(v) => v.to_string(),
        vr_ir::Literal::Float(v) => v.to_string(),
        vr_ir::Literal::Bool(v) => v.to_string(),
        vr_ir::Literal::Char(c) => format!("'{c}'"),
        vr_ir::Literal::Str(s) => format!("{s:?}"),
        vr_ir::Literal::Unit => "()".to_string(),
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p vr-editor view`
Expected: PASS — all four tests. If a `NodeKind` variant name mismatches (compile error), reconcile against `crates/graph/src/model.rs` (the `NodeKind` enum) and fix.

- [ ] **Step 5: Commit**

```bash
git add crates/editor/src/view.rs crates/editor/src/lib.rs
git commit -m "feat(editor): render model to egui_snarl (to_snarl + NodeView)"
```

---

### Task 5: eframe shell + read-only viewer (`app.rs`, `main.rs`) — human-validated

**Files:**
- Create: `C:\Projects\VisualRS\crates\editor\src\app.rs`
- Modify: `C:\Projects\VisualRS\crates\editor\src\main.rs`
- Modify: `C:\Projects\VisualRS\crates\editor\src\lib.rs`

**Interfaces:**
- Consumes: `seed::seed_graph`, `codegen::generate_source`, `view::{to_snarl, InputRow, NodeView}`, `eframe`, `egui`, `egui_snarl::{Snarl, SnarlViewer, InPin, OutPin, ui::{PinInfo, SnarlStyle}}`.
- Produces: `pub struct EditorApp` (`impl eframe::App`) and `pub fn run() -> eframe::Result<()>`.

**IMPORTANT — this task is human-validated, not unit-tested (ADR-0009).** The `SnarlViewer`/`Snarl::show`/`PinInfo`/`SnarlStyle` signatures below are the 0.11-shape reference; the first action is to confirm them against the installed crate and let the compiler guide fixes. There is no failing-test step here — the deliverable is a window a human eyeballs.

- [ ] **Step 1: Confirm the egui_snarl 0.11 viewer API**

Run: `cargo doc -p egui_snarl --no-deps --open` (or read `~/.cargo/registry/src/*/egui_snarl-0.11.0/src/`). Note the exact signatures of: `SnarlViewer::{title, inputs, outputs, show_input, show_output}` (in particular whether `show_input`/`show_output` take a `scale: f32` argument), `Snarl::show`, `PinInfo` constructors, and `SnarlStyle::new`. Adjust the code in Step 2 to match.

- [ ] **Step 2: Write the app shell and read-only viewer**

Create `C:\Projects\VisualRS\crates\editor\src\app.rs`:

```rust
//! The eframe shell. Thin and human-validated (not unit-tested): it holds the
//! derived read-only `Snarl` and the cached generated Rust, and lays out the
//! top menu, the canvas, and the live "Generated Rust" panel per the spec.

use eframe::egui;
use egui_snarl::ui::{PinInfo, SnarlStyle};
use egui_snarl::{InPin, OutPin, Snarl, SnarlViewer};

use crate::codegen::generate_source;
use crate::seed::seed_graph;
use crate::view::{to_snarl, InputRow, NodeView};

/// The whole editor: seed graph -> read-only `Snarl` + cached generated source.
/// The `Graph` is the source of truth; here it is consumed once at startup since
/// the skeleton is read-only (no canvas -> model adapter yet, per ADR-0009).
pub struct EditorApp {
    snarl: Snarl<NodeView>,
    generated: String,
    style: SnarlStyle,
}

impl Default for EditorApp {
    fn default() -> Self {
        let graph = seed_graph();
        let snarl = to_snarl(&graph);
        // generate_source never panics; on error it returns the message string,
        // which the panel shows verbatim.
        let generated = generate_source(&graph).unwrap_or_else(|e| e);
        Self {
            snarl,
            generated,
            style: SnarlStyle::new(),
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |_ui| {});
                ui.menu_button("Help", |_ui| {});
            });
        });

        egui::SidePanel::right("generated_rust")
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.heading("Generated Rust");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Read-only display: interactive(false) discards edits, so a
                    // per-frame clone as the buffer is fine for the skeleton.
                    let mut buf = self.generated.clone();
                    ui.add(
                        egui::TextEdit::multiline(&mut buf)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.snarl
                .show(&mut SkeletonViewer, &self.style, "vr_canvas", ui);
        });
    }
}

/// Read-only viewer: draws titles and pins from `NodeView`. All mutation hooks
/// are no-ops so the canvas cannot be edited (the model stays the source of
/// truth; editing arrives in a later phase).
struct SkeletonViewer;

impl SnarlViewer<NodeView> for SkeletonViewer {
    fn title(&mut self, node: &NodeView) -> String {
        node.title.clone()
    }

    fn inputs(&mut self, node: &NodeView) -> usize {
        node.inputs.len()
    }

    fn outputs(&mut self, node: &NodeView) -> usize {
        usize::from(node.has_output)
    }

    fn show_input(&mut self, pin: &InPin, ui: &mut egui::Ui, snarl: &mut Snarl<NodeView>) -> PinInfo {
        let node = &snarl[pin.id.node];
        match &node.inputs[pin.id.input] {
            InputRow::Wired { label } => {
                ui.label(label);
            }
            InputRow::Inline { text } => {
                ui.label(egui::RichText::new(text).monospace());
            }
        }
        PinInfo::circle()
    }

    fn show_output(&mut self, _pin: &OutPin, _ui: &mut egui::Ui, _snarl: &mut Snarl<NodeView>) -> PinInfo {
        PinInfo::circle()
    }

    // Read-only: refuse every mutation.
    fn connect(&mut self, _from: &OutPin, _to: &InPin, _snarl: &mut Snarl<NodeView>) {}
    fn disconnect(&mut self, _from: &OutPin, _to: &InPin, _snarl: &mut Snarl<NodeView>) {}
}

/// Boot the eframe window hosting the editor.
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Visual Rust",
        options,
        Box::new(|_cc| Ok(Box::new(EditorApp::default()))),
    )
}
```

Declare the module: add `pub mod app;` to `C:\Projects\VisualRS\crates\editor\src\lib.rs`.

- [ ] **Step 3: Wire the binary to the app**

Replace `C:\Projects\VisualRS\crates\editor\src\main.rs` with:

```rust
//! Binary entry point: boots the eframe window hosting the editor.

fn main() -> eframe::Result<()> {
    vr_editor::app::run()
}
```

- [ ] **Step 4: Build, fixing signatures against the compiler**

Run: `cargo build -p vr-editor`
Expected: PASS. If `show_input`/`show_output` require a `scale: f32` parameter in 0.11, or `Snarl::show` wants a different id-source type, or `PinInfo::circle()` is named differently, adjust to the confirmed 0.11 API (Step 1) until it compiles clean.

- [ ] **Step 5: Human validation — run the app and eyeball it**

Run: `cargo run -p vr-editor`
Expected (visually confirm):
- A window titled "Visual Rust" opens with a top menu bar (`File`, `Help` — inert).
- The central canvas shows 4 nodes: an `Add` node with inline `1` and `2`, a `let n` node wired from `Add`, a `println!("n: {}")` node with inline `n`, and an `expr` node wired from `println!`. Two data wires are visible.
- The right panel "Generated Rust" shows monospaced, read-only:
  ```
  fn main() {
      let n = (1 + 2);
      println!("n: {}", n);
  }
  ```
- Dragging pins does nothing (read-only); the app does not panic.

- [ ] **Step 6: Commit**

```bash
git add crates/editor/src/app.rs crates/editor/src/main.rs crates/editor/src/lib.rs
git commit -m "feat(editor): eframe shell with read-only canvas and live Rust panel"
```

---

## Final Verification (whole workspace)

Run each from `C:\Projects\VisualRS` and confirm green before considering the skeleton done (spec §8 Definition of Done):

- [ ] `cargo fmt --check` — PASS (formatting gate).
- [ ] `cargo clippy --all-targets -- -D warnings` — PASS (no warnings anywhere, including `vr-editor`).
- [ ] `cargo build` — PASS (whole workspace).
- [ ] `cargo test` — PASS (all crates; `vr-editor`'s `seed`/`codegen`/`view` conformance tests included).
- [ ] `cargo run -p vr-editor` — window matches the Task 5 Step 5 checklist (human-validated).
- [ ] Layering gate: `cargo tree -p vr-editor -i syn` and `-i proc-macro2` show them only under `vr-rustgen`, never as direct deps; `cargo tree -p vr-editor -i gdext` reports no match (`godot`/`gdext` absent).

## Notes for the executor

- Follow the CLAUDE.md commit rules: Conventional Commits, commit locally per task, **never `git push`** without Felipe's explicit go-ahead, no AI attribution, no emojis.
- Do not add `syn`/`quote`/`proc-macro2` or `godot`/`gdext` to `vr-editor` to "make something work" — if you feel the need, stop; that is an ADR-0009 layering violation and a design smell.
- The spec is the documented behavior (ADR-0012 DDD). If reality forces a divergence (e.g. an API differs from what this plan assumed), stop and surface it rather than silently reshaping the design.
