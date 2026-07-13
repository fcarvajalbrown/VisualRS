# Phase 2 (headless core): vr-graph model + graph -> IR lowering — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a pure-Rust `vr-graph` crate — a Blueprints-style node/pin/wire graph model, a `lower()` that compiles it to a `vr_ir::Program`, and graph validation — proven by lowering a hand-built capstone graph to byte-identical source with (and a compile+run matching) the Phase 1 `line_report` fixture.

**Architecture:** `vr-graph` is a front-end that depends on `vr-ir` only and emits a `vr_ir::Program`; the existing `vr-rustgen` backend consumes it unchanged (`vr-graph -> vr-ir -> vr-rustgen`). Structs/enums are declarative decls; function bodies are `Block`s of nodes threaded by exec edges, with value nodes connected by typed data edges and inline leaves for trivial constants. The eventual Godot `gdext` plugin (deferred) will wrap this crate.

**Tech Stack:** Rust (edition 2021), `vr-ir` (no `syn`/`quote`, no `godot`/`gdext` in this crate), `vr-rustgen` + `rustc` used only in tests.

## Global Constraints

- **Target-agnostic IR (ADR-0005):** `vr-graph` depends on `vr-ir` only. It MUST NOT depend on `syn`/`quote`/`proc-macro2` (those stay in `vr-rustgen`) and MUST NOT depend on `godot`/`gdext` (the plugin is deferred).
- **Reuse IR vocabulary:** reuse `vr_ir::{Type, Literal, BinaryOp, AssignOp, Pattern, VariantPayload}` — do not define parallel enums.
- **Edition/MSRV:** edition 2021, `rust-version` inherited from the workspace (`1.94`).
- **Success oracle:** the graph-derived program must generate source byte-identical to `vr_rustgen::generate(&vr_ir::fixtures::line_report())`, and compile+run with the same counts (`lines: 4, blank: 1, comment: 1, content: 2, words: 5`). If exact parity forces an unnatural graph shape, STOP and ask Felipe rather than distorting the model.
- **Repo conventions (`CLAUDE.md`):** no emojis anywhere; no AI attribution; Conventional Commits (`feat(graph): ...`); never open a PR unless explicitly asked. Before committing, run `cargo fmt --check` and `cargo clippy -p vr-graph --all-targets -- -D warnings`.
- **Roadmap:** Phase 2 stays **In Progress** after this work (only the headless core lands; the Godot plugin is deferred).

---

## File Structure

```
Cargo.toml                        # add crates/graph to workspace members + workspace dep
crates/graph/
  Cargo.toml                      # vr-graph: dep on vr-ir; dev-deps vr-rustgen
  src/
    lib.rs                        # re-exports + crate docs
    model.rs                      # NodeId, Leaf, DataEdge, Node, NodeKind, Block, Arm, decls, Graph, GraphItem, FunctionGraph
    build.rs                      # BlockBuilder + Src value helpers (ergonomics for fixtures/tests)
    lower.rs                      # lower(&Graph) -> Result<vr_ir::Program, LowerError>
    validate.rs                   # Graph::validate() -> Result<(), Vec<String>>
    fixtures.rs                   # line_report_graph(): the capstone parity fixture
  tests/
    compile.rs                    # compile + run the graph-derived program (parity counts)
docs/adr/0008-vr-graph-headless-front-end.md   # ADR for the front-end layering
docs/adr/README.md                # index row for 0008
ROADMAP.md                        # Phase 2 status note (still In Progress)
```

---

### Task 1: Scaffold the `vr-graph` crate

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Create: `crates/graph/Cargo.toml`, `crates/graph/src/lib.rs`

**Interfaces:**
- Consumes: `vr-ir` (path dep), `vr-rustgen` (dev-dep, for parity tests).
- Produces: a compiling `vr_graph` lib crate in the workspace.

- [ ] **Step 1: Add the crate to the workspace**

In root `Cargo.toml`, change the members line to include the new crate:

```toml
members = ["crates/ir", "crates/rustgen", "crates/cli", "crates/graph"]
```

And add to `[workspace.dependencies]` (below the existing `vr-rustgen` line):

```toml
vr-graph = { path = "crates/graph" }
```

- [ ] **Step 2: Write the crate manifest**

Create `crates/graph/Cargo.toml`:

```toml
[package]
name = "vr-graph"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Visual Rust: Blueprints-style graph model and graph -> IR lowering"

[lib]
name = "vr_graph"

[dependencies]
vr-ir.workspace = true
# No syn/quote/proc-macro2 (ADR-0005). No godot/gdext (plugin is deferred).

[dev-dependencies]
vr-rustgen.workspace = true
vr-ir.workspace = true
```

- [ ] **Step 3: Write the crate root**

Create `crates/graph/src/lib.rs`:

```rust
//! Visual Rust graph front-end: a Blueprints-style node/pin/wire model of a
//! program, plus lowering to the target-agnostic `vr_ir`. Depends on `vr-ir`
//! only; it knows nothing about Rust syntax (that is `vr-rustgen`) or Godot
//! (that is the deferred editor plugin). See
//! `docs/superpowers/specs/2026-07-12-phase2-headless-graph-core-design.md`.

pub mod model;

pub use model::{
    Arm, Block, DataEdge, EnumDecl, FunctionGraph, Graph, GraphItem, Leaf, Node, NodeId, NodeKind,
    StructDecl, VariantDecl,
};
```

- [ ] **Step 4: Verify the workspace builds**

Run: `cargo build -p vr-graph`
Expected: FAIL — `model` module file does not exist yet. (That is fine; Task 2 creates it. If you prefer a green checkpoint here, temporarily comment the `pub mod model;` line and re-run; then restore it in Task 2.)

- [ ] **Step 5: Commit** (after Task 2 makes it build; or commit the manifest+members now with `model` commented, your choice)

```bash
git add Cargo.toml crates/graph/Cargo.toml crates/graph/src/lib.rs
git commit -m "chore(graph): scaffold vr-graph crate in the workspace"
```

---

### Task 2: The graph model

**Files:**
- Create: `crates/graph/src/model.rs`
- Modify: `crates/graph/src/lib.rs` (re-exports already added in Task 1)

**Interfaces:**
- Consumes: `vr_ir::{Type, Literal, BinaryOp, AssignOp, Pattern, VariantPayload}`.
- Produces:
  - `NodeId(u32)` (Copy, Ord, Hash).
  - `Leaf` enum: `Lit(Literal)`, `Var(String)`, `Path(Vec<String>)`.
  - `DataEdge { from: NodeId, to: NodeId, to_port: u16 }` — a value node's single output feeds `to`'s input port `to_port`.
  - `Node { kind: NodeKind, inline: Vec<(u16, Leaf)> }`.
  - `NodeKind` (statement + value variants, see code).
  - `Block { nodes: BTreeMap<NodeId, Node>, exec: Vec<(NodeId, NodeId)>, data: Vec<DataEdge>, entry: Option<NodeId>, tail: Option<NodeId> }`.
  - `Arm { pattern: Pattern, guard: Option<Block>, body: Block }`.
  - decls: `StructDecl`, `EnumDecl`, `VariantDecl`, `FunctionGraph`.
  - `Graph { items: Vec<GraphItem> }`, `GraphItem` enum.

Port conventions (used by `build.rs` and `lower.rs`): `Let` in0=value; `Assign` in0=target, in1=value; `ForEach` in0=iter; `ExprStmt` in0=expr; `Return` in0=value; `Field` in0=base; `Call` in0=func, in1..=args; `Method` in0=receiver, in1..=args; `Binary` in0=lhs, in1=rhs; `Ref` in0=value; `StructLit` in_i=field i; `Builtin` in0..=args; `Try` in0=value; `Match` in0=scrutinee; `If` in0=cond. Every value node has exactly one output.

- [ ] **Step 1: Write the failing test**

Add to the bottom of `crates/graph/src/model.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use vr_ir::Literal;

    #[test]
    fn block_holds_nodes_and_edges() {
        let mut nodes = std::collections::BTreeMap::new();
        nodes.insert(
            NodeId(0),
            Node { kind: NodeKind::Let { name: "n".into(), mutable: false }, inline: vec![(0, Leaf::Lit(Literal::Int(1)))] },
        );
        let block = Block {
            nodes,
            exec: vec![],
            data: vec![],
            entry: Some(NodeId(0)),
            tail: None,
        };
        assert_eq!(block.entry, Some(NodeId(0)));
        assert!(matches!(block.nodes[&NodeId(0)].kind, NodeKind::Let { .. }));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-graph`
Expected: FAIL — `NodeKind`/`Block` not found.

- [ ] **Step 3: Write the model types**

Prepend to `crates/graph/src/model.rs` (above the test module):

```rust
use std::collections::BTreeMap;
use vr_ir::{AssignOp, BinaryOp, Literal, Pattern, Type, VariantPayload};

/// A stable identity for a node within a single `Block`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u32);

/// An inline value on an input pin: the "default value widget" of a Blueprints
/// input port. Used for trivial leaves instead of a wired node.
#[derive(Clone, Debug, PartialEq)]
pub enum Leaf {
    Lit(Literal),
    /// A binding reference by name (a "Get" node), e.g. `report`, `line`.
    Var(String),
    /// A path constant, e.g. `LineKind::Blank`, `Ok`, `None`.
    Path(Vec<String>),
}

/// A data wire: the single output of `from` feeds input port `to_port` of `to`.
#[derive(Clone, Debug, PartialEq)]
pub struct DataEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub to_port: u16,
}

/// A node plus any inline leaf values for input ports that are not wired.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub inline: Vec<(u16, Leaf)>,
}

/// Statement/control nodes (threaded by exec edges) and value nodes (connected
/// by data edges). See the port conventions in this task's header.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind {
    // --- statement / control (threaded by `Block::exec`) ---
    Let { name: String, mutable: bool },
    Assign { op: AssignOp },
    ForEach { binding: String, body: Block },
    ExprStmt,
    Return { has_value: bool },
    // --- value (connected by `Block::data` / inline leaves) ---
    Field { name: String },
    Call,
    Method { method: String },
    Binary { op: BinaryOp },
    Ref { mutable: bool },
    StructLit { name: String, fields: Vec<String> },
    Builtin { op: vr_ir::BuiltinOp },
    Try,
    Match { arms: Vec<Arm> },
    If { then: Block, els: Option<Block> },
}

/// A braced scope: statement nodes threaded from `entry` via `exec`, value nodes
/// wired via `data`, and an optional trailing value node `tail`.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub nodes: BTreeMap<NodeId, Node>,
    /// Sequential exec thread: `(from_stmt, to_stmt)` pairs.
    pub exec: Vec<(NodeId, NodeId)>,
    pub data: Vec<DataEdge>,
    pub entry: Option<NodeId>,
    pub tail: Option<NodeId>,
}

/// One arm of a `Match` value node. `body`/`guard` are sub-blocks.
#[derive(Clone, Debug, PartialEq)]
pub struct Arm {
    pub pattern: Pattern,
    pub guard: Option<Block>,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariantDecl {
    pub name: String,
    pub payload: VariantPayload,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<VariantDecl>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionGraph {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret: Type,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GraphItem {
    Struct(StructDecl),
    Enum(EnumDecl),
    Function(FunctionGraph),
}

/// A whole program as a graph: declarative struct/enum decls plus function
/// bodies as node graphs.
#[derive(Clone, Debug, PartialEq)]
pub struct Graph {
    pub items: Vec<GraphItem>,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vr-graph`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/graph
git commit -m "feat(graph): add Blueprints-style graph model types"
```

---

### Task 3: Block builder and value helpers

**Files:**
- Create: `crates/graph/src/build.rs`
- Modify: `crates/graph/src/lib.rs`

**Interfaces:**
- Consumes: `model` types.
- Produces:
  - `Src` enum: `Node(NodeId)`, `Leaf(Leaf)` — a resolved input source.
  - `BlockBuilder` with: `new()`, `value(kind) -> NodeId`, `feed(node, port, Src)`, `stmt(kind) -> NodeId`, `set_tail(NodeId)`, `build() -> Block`.
  - free helpers returning `Src`: `lit(Literal)`, `int(i128)`, `var(&str)`, `path(&[&str])` and node-building helpers that take `&mut BlockBuilder`.

These exist so fixtures and tests read cleanly instead of hand-assembling `BTreeMap`s and port indices.

- [ ] **Step 1: Write the failing test**

Add to the bottom of `crates/graph/src/build.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use vr_ir::BinaryOp;

    #[test]
    fn builds_a_block_with_a_wired_binary_tail() {
        // { 1 + 2 }  (as a tail value)
        let mut b = BlockBuilder::new();
        let sum = binary(&mut b, BinaryOp::Add, int(1), int(2));
        b.set_tail_src(sum);
        let block = b.build();
        assert_eq!(block.nodes.len(), 1); // the Binary node; operands are inline leaves
        assert!(block.tail.is_some());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-graph build`
Expected: FAIL — `BlockBuilder`/`binary` not found.

- [ ] **Step 3: Write the builder**

Prepend to `crates/graph/src/build.rs`:

```rust
use std::collections::BTreeMap;
use vr_ir::{BinaryOp, BuiltinOp, Literal};

use crate::model::{Block, DataEdge, Leaf, Node, NodeId, NodeKind};

/// A resolved input source: either a wired node output or an inline leaf.
#[derive(Clone, Debug)]
pub enum Src {
    Node(NodeId),
    Leaf(Leaf),
}

/// Incrementally assembles a `Block`, assigning `NodeId`s and threading exec.
pub struct BlockBuilder {
    nodes: BTreeMap<NodeId, Node>,
    exec: Vec<(NodeId, NodeId)>,
    data: Vec<DataEdge>,
    entry: Option<NodeId>,
    last_stmt: Option<NodeId>,
    tail: Option<NodeId>,
    next: u32,
}

impl BlockBuilder {
    pub fn new() -> Self {
        BlockBuilder {
            nodes: BTreeMap::new(),
            exec: vec![],
            data: vec![],
            entry: None,
            last_stmt: None,
            tail: None,
            next: 0,
        }
    }

    fn fresh(&mut self) -> NodeId {
        let id = NodeId(self.next);
        self.next += 1;
        id
    }

    /// Insert a value node (no exec threading) and return its id.
    pub fn value(&mut self, kind: NodeKind) -> NodeId {
        let id = self.fresh();
        self.nodes.insert(id, Node { kind, inline: vec![] });
        id
    }

    /// Insert a statement node and thread it onto the exec chain.
    pub fn stmt(&mut self, kind: NodeKind) -> NodeId {
        let id = self.fresh();
        self.nodes.insert(id, Node { kind, inline: vec![] });
        match self.last_stmt {
            None => self.entry = Some(id),
            Some(prev) => self.exec.push((prev, id)),
        }
        self.last_stmt = Some(id);
        id
    }

    /// Feed input port `port` of `node` from `src` (a wire or an inline leaf).
    pub fn feed(&mut self, node: NodeId, port: u16, src: Src) {
        match src {
            Src::Node(from) => self.data.push(DataEdge { from, to: node, to_port: port }),
            Src::Leaf(leaf) => {
                if let Some(n) = self.nodes.get_mut(&node) {
                    n.inline.push((port, leaf));
                }
            }
        }
    }

    /// Feed a sequence of ports 0..n of `node` from `srcs` in order.
    pub fn feed_all(&mut self, node: NodeId, srcs: Vec<Src>) {
        for (i, s) in srcs.into_iter().enumerate() {
            self.feed(node, i as u16, s);
        }
    }

    pub fn set_tail(&mut self, id: NodeId) {
        self.tail = Some(id);
    }

    /// Convenience: set the tail from a `Src`. A leaf tail is materialised as a
    /// node-free tail by wrapping it in a passthrough is unnecessary here because
    /// tails in the capstone are always node values or bare leaves handled by
    /// lowering; for a leaf tail we insert a `Ref`-free identity via a Var/Path
    /// node is not needed — instead store the leaf on a synthetic single-input
    /// `ExprStmt`? No: for a leaf tail, callers use a value node. This helper
    /// only accepts node sources; leaf tails are represented by their own node.
    pub fn set_tail_src(&mut self, src: Src) {
        match src {
            Src::Node(id) => self.tail = Some(id),
            Src::Leaf(_) => panic!("tail must be a value node; wrap leaves in a value node"),
        }
    }

    pub fn build(self) -> Block {
        Block {
            nodes: self.nodes,
            exec: self.exec,
            data: self.data,
            entry: self.entry,
            tail: self.tail,
        }
    }
}

impl Default for BlockBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// --- leaf value helpers (return Src::Leaf) ----------------------------------
pub fn lit(l: Literal) -> Src {
    Src::Leaf(Leaf::Lit(l))
}
pub fn int(v: i128) -> Src {
    Src::Leaf(Leaf::Lit(Literal::Int(v)))
}
pub fn var(name: &str) -> Src {
    Src::Leaf(Leaf::Var(name.into()))
}
pub fn path(segs: &[&str]) -> Src {
    Src::Leaf(Leaf::Path(segs.iter().map(|s| s.to_string()).collect()))
}

// --- node value helpers (insert a node, wire inputs, return Src::Node) -------
pub fn field(b: &mut BlockBuilder, base: Src, name: &str) -> Src {
    let id = b.value(NodeKind::Field { name: name.into() });
    b.feed(id, 0, base);
    Src::Node(id)
}
pub fn method(b: &mut BlockBuilder, receiver: Src, m: &str, args: Vec<Src>) -> Src {
    let id = b.value(NodeKind::Method { method: m.into() });
    b.feed(id, 0, receiver);
    for (i, a) in args.into_iter().enumerate() {
        b.feed(id, (i + 1) as u16, a);
    }
    Src::Node(id)
}
pub fn call(b: &mut BlockBuilder, func: Src, args: Vec<Src>) -> Src {
    let id = b.value(NodeKind::Call);
    b.feed(id, 0, func);
    for (i, a) in args.into_iter().enumerate() {
        b.feed(id, (i + 1) as u16, a);
    }
    Src::Node(id)
}
pub fn binary(b: &mut BlockBuilder, op: BinaryOp, lhs: Src, rhs: Src) -> Src {
    let id = b.value(NodeKind::Binary { op });
    b.feed(id, 0, lhs);
    b.feed(id, 1, rhs);
    Src::Node(id)
}
pub fn reference(b: &mut BlockBuilder, mutable: bool, value: Src) -> Src {
    let id = b.value(NodeKind::Ref { mutable });
    b.feed(id, 0, value);
    Src::Node(id)
}
pub fn struct_lit(b: &mut BlockBuilder, name: &str, fields: Vec<(&str, Src)>) -> Src {
    let names: Vec<String> = fields.iter().map(|(n, _)| n.to_string()).collect();
    let id = b.value(NodeKind::StructLit { name: name.into(), fields: names });
    for (i, (_, s)) in fields.into_iter().enumerate() {
        b.feed(id, i as u16, s);
    }
    Src::Node(id)
}
pub fn builtin(b: &mut BlockBuilder, op: BuiltinOp, args: Vec<Src>) -> Src {
    let id = b.value(NodeKind::Builtin { op });
    for (i, a) in args.into_iter().enumerate() {
        b.feed(id, i as u16, a);
    }
    Src::Node(id)
}
pub fn try_(b: &mut BlockBuilder, value: Src) -> Src {
    let id = b.value(NodeKind::Try);
    b.feed(id, 0, value);
    Src::Node(id)
}
```

Add to `crates/graph/src/lib.rs`:

```rust
pub mod build;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vr-graph build`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/graph
git commit -m "feat(graph): add block builder and value helpers"
```

---

### Task 4: Lowering to `vr_ir::Program`

**Files:**
- Create: `crates/graph/src/lower.rs`
- Modify: `crates/graph/src/lib.rs`

**Interfaces:**
- Consumes: `model` types; produces `vr_ir` AST.
- Produces:
  - `pub fn lower(graph: &Graph) -> Result<vr_ir::Program, LowerError>`
  - `pub enum LowerError { MissingInput { port: u16 }, MissingEntry, DanglingNode(NodeId), NoTailValue }` (+ `Display`).

Lowering rules (parity-critical):
- `lower_block(block) -> vr_ir::Block`: walk exec from `entry`, lower each statement node in order; `tail = block.tail.map(lower_value)`.
- `lower_value(block, id) -> Expr`: match the value node kind; variadic nodes (`Call`/`Method`/`Builtin`/`StructLit`) collect inputs `0..input_count`.
- `lower_arm_body(block) -> Expr`: if the arm block has no statements (`entry` is `None`) and a tail, lower the tail bare; else `Expr::Block(lower_block(block))`. (Match arms allow bare exprs.)
- `lower_else(block) -> Expr`: if the else block has no statements and its tail node is itself an `If`, lower it bare (an `else if` chain); else `Expr::Block(lower_block(block))`. (A Rust `else` must be a block or an `if`.)
- `If.then` lowers to a `vr_ir::Block` directly (never wrapped).

- [ ] **Step 1: Write the failing test**

Add to the bottom of `crates/graph/src/lower.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::*;
    use crate::model::*;
    use vr_ir::{BinaryOp, BuiltinOp, Type};

    // fn main() { let n = 1 + 2; println!("n: {}", n); }
    fn arithmetic_graph() -> Graph {
        let mut b = BlockBuilder::new();
        let sum = binary(&mut b, BinaryOp::Add, int(1), int(2));
        let let_n = b.stmt(NodeKind::Let { name: "n".into(), mutable: false });
        b.feed(let_n, 0, sum);
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

    #[test]
    fn arithmetic_lowers_and_generates() {
        let prog = lower(&arithmetic_graph()).expect("lowers");
        let src = vr_rustgen::generate(&prog).expect("generates");
        assert!(src.contains("let n = (1 + 2)"), "got:\n{src}");
        assert!(src.contains(r#"println!("n: {}", n)"#), "got:\n{src}");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-graph lower`
Expected: FAIL — `lower` not found.

- [ ] **Step 3: Write the lowering**

Prepend to `crates/graph/src/lower.rs`:

```rust
use std::fmt;

use vr_ir::{
    Block as IrBlock, EnumDef, Expr, Field as IrField, FunctionDef, Item, MatchArm, Param, Program,
    Stmt, StructDef, Variant,
};

use crate::model::{
    Arm, Block, GraphItem, Graph, Leaf, NodeId, NodeKind,
};

/// A lowering failure. A validated graph should never produce these.
#[derive(Clone, Debug, PartialEq)]
pub enum LowerError {
    MissingInput { port: u16 },
    MissingEntry,
    DanglingNode(NodeId),
    NoTailValue,
}

impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LowerError::MissingInput { port } => write!(f, "input port {port} has no source"),
            LowerError::MissingEntry => write!(f, "block has statements-shaped exec but no entry"),
            LowerError::DanglingNode(n) => write!(f, "node {n:?} referenced but not present"),
            LowerError::NoTailValue => write!(f, "expected a tail value but found none"),
        }
    }
}

impl std::error::Error for LowerError {}

/// Lower a whole graph to a `vr_ir::Program`.
pub fn lower(graph: &Graph) -> Result<Program, LowerError> {
    let mut items = Vec::new();
    for item in &graph.items {
        items.push(match item {
            GraphItem::Struct(s) => Item::Struct(StructDef {
                name: s.name.clone(),
                fields: s
                    .fields
                    .iter()
                    .map(|(n, t)| IrField { name: n.clone(), ty: t.clone() })
                    .collect(),
            }),
            GraphItem::Enum(e) => Item::Enum(EnumDef {
                name: e.name.clone(),
                variants: e
                    .variants
                    .iter()
                    .map(|v| Variant { name: v.name.clone(), payload: v.payload.clone() })
                    .collect(),
            }),
            GraphItem::Function(func) => Item::Function(FunctionDef {
                name: func.name.clone(),
                params: func
                    .params
                    .iter()
                    .map(|(n, t)| Param { name: n.clone(), ty: t.clone() })
                    .collect(),
                ret: func.ret.clone(),
                body: lower_block(&func.body)?,
            }),
        });
    }
    Ok(Program { items })
}

/// Follow the exec thread from `entry`, lowering statements in order.
fn lower_block(block: &Block) -> Result<IrBlock, LowerError> {
    let mut stmts = Vec::new();
    let mut cur = block.entry;
    while let Some(id) = cur {
        stmts.push(lower_stmt(block, id)?);
        cur = next_exec(block, id);
    }
    let tail = match block.tail {
        Some(t) => Some(lower_value(block, t)?),
        None => None,
    };
    Ok(IrBlock { stmts, tail: tail.map(Box::new) })
}

fn next_exec(block: &Block, from: NodeId) -> Option<NodeId> {
    block.exec.iter().find(|(a, _)| *a == from).map(|(_, b)| *b)
}

fn lower_stmt(block: &Block, id: NodeId) -> Result<Stmt, LowerError> {
    let node = block.nodes.get(&id).ok_or(LowerError::DanglingNode(id))?;
    Ok(match &node.kind {
        NodeKind::Let { name, mutable } => Stmt::Let {
            name: name.clone(),
            mutable: *mutable,
            ty: None,
            value: resolve(block, id, 0)?,
        },
        NodeKind::Assign { op } => Stmt::Assign {
            target: resolve(block, id, 0)?,
            op: *op,
            value: resolve(block, id, 1)?,
        },
        NodeKind::ForEach { binding, body } => Stmt::ForEach {
            binding: binding.clone(),
            iter: resolve(block, id, 0)?,
            body: lower_block(body)?,
        },
        NodeKind::ExprStmt => Stmt::Expr(resolve(block, id, 0)?),
        NodeKind::Return { has_value } => {
            if *has_value {
                Stmt::Return(Some(resolve(block, id, 0)?))
            } else {
                Stmt::Return(None)
            }
        }
        _ => return Err(LowerError::DanglingNode(id)), // a value node used as a statement
    })
}

/// Resolve input `port` of `node`: a data edge lowers its source value; else the
/// inline leaf lowers; else the input is missing.
fn resolve(block: &Block, node: NodeId, port: u16) -> Result<Expr, LowerError> {
    if let Some(edge) = block.data.iter().find(|e| e.to == node && e.to_port == port) {
        return lower_value(block, edge.from);
    }
    let n = block.nodes.get(&node).ok_or(LowerError::DanglingNode(node))?;
    if let Some((_, leaf)) = n.inline.iter().find(|(p, _)| *p == port) {
        return Ok(lower_leaf(leaf));
    }
    Err(LowerError::MissingInput { port })
}

fn lower_leaf(leaf: &Leaf) -> Expr {
    match leaf {
        Leaf::Lit(l) => Expr::Lit(l.clone()),
        Leaf::Var(n) => Expr::Var(n.clone()),
        Leaf::Path(segs) => Expr::Path(segs.clone()),
    }
}

/// Highest input port index (+1) that has any source on `node`.
fn input_count(block: &Block, node: NodeId) -> u16 {
    let mut max: i32 = -1;
    for e in block.data.iter().filter(|e| e.to == node) {
        max = max.max(e.to_port as i32);
    }
    if let Some(n) = block.nodes.get(&node) {
        for (p, _) in &n.inline {
            max = max.max(*p as i32);
        }
    }
    (max + 1) as u16
}

fn resolve_variadic(block: &Block, node: NodeId, start: u16) -> Result<Vec<Expr>, LowerError> {
    let count = input_count(block, node);
    let mut out = Vec::new();
    for p in start..count {
        out.push(resolve(block, node, p)?);
    }
    Ok(out)
}

fn lower_value(block: &Block, id: NodeId) -> Result<Expr, LowerError> {
    let node = block.nodes.get(&id).ok_or(LowerError::DanglingNode(id))?;
    Ok(match &node.kind {
        NodeKind::Field { name } => Expr::Field {
            base: Box::new(resolve(block, id, 0)?),
            name: name.clone(),
        },
        NodeKind::Call => {
            let func = resolve(block, id, 0)?;
            let args = resolve_variadic(block, id, 1)?;
            Expr::Call { func: Box::new(func), args }
        }
        NodeKind::Method { method } => {
            let receiver = resolve(block, id, 0)?;
            let args = resolve_variadic(block, id, 1)?;
            Expr::MethodCall { receiver: Box::new(receiver), method: method.clone(), args }
        }
        NodeKind::Binary { op } => Expr::Binary {
            op: *op,
            lhs: Box::new(resolve(block, id, 0)?),
            rhs: Box::new(resolve(block, id, 1)?),
        },
        NodeKind::Ref { mutable } => Expr::Ref {
            mutable: *mutable,
            expr: Box::new(resolve(block, id, 0)?),
        },
        NodeKind::StructLit { name, fields } => {
            let mut out = Vec::new();
            for (i, fname) in fields.iter().enumerate() {
                out.push((fname.clone(), resolve(block, id, i as u16)?));
            }
            Expr::StructLit { name: name.clone(), fields: out }
        }
        NodeKind::Builtin { op } => {
            let args = resolve_variadic(block, id, 0)?;
            Expr::Builtin { op: op.clone(), args }
        }
        NodeKind::Try => Expr::Try(Box::new(resolve(block, id, 0)?)),
        NodeKind::Match { arms } => {
            let scrutinee = resolve(block, id, 0)?;
            let mut ir_arms = Vec::new();
            for a in arms {
                ir_arms.push(lower_arm(block, a)?);
            }
            Expr::Match { scrutinee: Box::new(scrutinee), arms: ir_arms }
        }
        NodeKind::If { then, els } => {
            let cond = resolve(block, id, 0)?;
            let then_block = lower_block(then)?;
            let else_expr = match els {
                Some(eb) => Some(Box::new(lower_else(eb)?)),
                None => None,
            };
            Expr::If { cond: Box::new(cond), then: then_block, else_: else_expr }
        }
        // statement kinds are not values
        NodeKind::Let { .. }
        | NodeKind::Assign { .. }
        | NodeKind::ForEach { .. }
        | NodeKind::ExprStmt
        | NodeKind::Return { .. } => return Err(LowerError::DanglingNode(id)),
    })
}

fn lower_arm(block: &Block, arm: &Arm) -> Result<MatchArm, LowerError> {
    let guard = match &arm.guard {
        Some(g) => Some(lower_arm_body(g)?),
        None => None,
    };
    Ok(MatchArm {
        pattern: arm.pattern.clone(),
        guard,
        body: lower_arm_body(&arm.body)?,
    })
}

/// A match-arm body: bare tail if there are no statements, else a block expr.
fn lower_arm_body(block: &Block) -> Result<Expr, LowerError> {
    if block.entry.is_none() {
        if let Some(t) = block.tail {
            return lower_value(block, t);
        }
    }
    Ok(Expr::Block(lower_block(block)?))
}

/// An `else` branch: bare only if it is an `else if` (no stmts, tail is an `If`).
fn lower_else(block: &Block) -> Result<Expr, LowerError> {
    if block.entry.is_none() {
        if let Some(t) = block.tail {
            if let Some(n) = block.nodes.get(&t) {
                if matches!(n.kind, NodeKind::If { .. }) {
                    return lower_value(block, t);
                }
            }
        }
    }
    Ok(Expr::Block(lower_block(block)?))
}
```

Add to `crates/graph/src/lib.rs`:

```rust
pub mod lower;

pub use lower::{lower, LowerError};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vr-graph lower`
Expected: PASS — the arithmetic graph lowers and generates the expected source.

- [ ] **Step 5: Commit**

```bash
git add crates/graph
git commit -m "feat(graph): lower graph model to vr_ir Program"
```

---

### Task 5: Graph validation

**Files:**
- Create: `crates/graph/src/validate.rs`
- Modify: `crates/graph/src/lib.rs`

**Interfaces:**
- Consumes: `model` types.
- Produces: `impl Graph { pub fn validate(&self) -> Result<(), Vec<String>> }`.

Phase-2-core checks (collect all problems):
- exactly one function named `main`; no two items share a name.
- every `Type::Named(n)` in a struct field, enum payload, function param, or return refers to a declared struct/enum.
- within every function body (recursively into `ForEach` bodies, `Match` arms, `If` branches): every required input port of every reachable node is satisfied by a data edge or an inline leaf (missing-input detection).
- basic pin typing where statically knowable: a `StructLit` field fed by an inline `Lit` whose literal kind is incompatible with the declared field type is rejected (e.g. a bool literal into a `usize` field). Unknown types (method/call outputs) are not rejected — richer pin inference needs the stdlib type table and is deferred to Phase 4.

- [ ] **Step 1: Write the failing test**

Create `crates/graph/src/validate.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::build::*;
    use crate::model::*;
    use vr_ir::{Literal, Type};

    fn empty_main() -> FunctionGraph {
        FunctionGraph {
            name: "main".into(),
            params: vec![],
            ret: Type::Unit,
            body: BlockBuilder::new().build(),
        }
    }

    #[test]
    fn well_formed_graph_validates() {
        let g = Graph { items: vec![GraphItem::Function(empty_main())] };
        assert!(g.validate().is_ok());
    }

    #[test]
    fn missing_main_is_reported() {
        let g = Graph { items: vec![] };
        let errs = g.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("main")), "got: {errs:?}");
    }

    #[test]
    fn undefined_named_type_is_reported() {
        let g = Graph {
            items: vec![GraphItem::Function(FunctionGraph {
                name: "main".into(),
                params: vec![("r".into(), Type::Named("Ghost".into()))],
                ret: Type::Unit,
                body: BlockBuilder::new().build(),
            })],
        };
        let errs = g.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("Ghost")), "got: {errs:?}");
    }

    #[test]
    fn struct_field_type_mismatch_is_reported() {
        // Report { words: usize } but the graph feeds a bool literal.
        let mut b = BlockBuilder::new();
        let lit_node = struct_lit(&mut b, "Report", vec![("words", lit(Literal::Bool(true)))]);
        let let_r = b.stmt(NodeKind::Let { name: "r".into(), mutable: false });
        b.feed(let_r, 0, lit_node);
        let g = Graph {
            items: vec![
                GraphItem::Struct(StructDecl {
                    name: "Report".into(),
                    fields: vec![("words".into(), Type::Usize)],
                }),
                GraphItem::Function(FunctionGraph {
                    name: "main".into(),
                    params: vec![],
                    ret: Type::Unit,
                    body: b.build(),
                }),
            ],
        };
        let errs = g.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("words")), "got: {errs:?}");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-graph validate`
Expected: FAIL — `validate` not found.

- [ ] **Step 3: Write the validator**

Prepend to `crates/graph/src/validate.rs`:

```rust
use std::collections::HashSet;

use vr_ir::{Literal, Type, VariantPayload};

use crate::model::{Block, Graph, GraphItem, Leaf, Node, NodeId, NodeKind};

impl Graph {
    /// Phase-2 well-formedness. Returns all problems found, or `Ok`.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Declared type names + item-name/main checks.
        let mut defined: HashSet<&str> = HashSet::new();
        let mut names: HashSet<String> = HashSet::new();
        let mut main_count = 0usize;
        let mut struct_fields: Vec<(&str, &Vec<(String, Type)>)> = Vec::new();
        for item in &self.items {
            let name = match item {
                GraphItem::Struct(s) => {
                    defined.insert(s.name.as_str());
                    struct_fields.push((s.name.as_str(), &s.fields));
                    s.name.clone()
                }
                GraphItem::Enum(e) => {
                    defined.insert(e.name.as_str());
                    e.name.clone()
                }
                GraphItem::Function(f) => {
                    if f.name == "main" {
                        main_count += 1;
                    }
                    f.name.clone()
                }
            };
            if !names.insert(name.clone()) {
                errors.push(format!("duplicate top-level item name: `{name}`"));
            }
        }
        if main_count == 0 {
            errors.push("no entry point: expected a function named `main`".into());
        } else if main_count > 1 {
            errors.push(format!("expected exactly one `main`, found {main_count}"));
        }

        // Named-type references across item signatures.
        for item in &self.items {
            match item {
                GraphItem::Struct(s) => {
                    for (_, t) in &s.fields {
                        check_named(t, &defined, &mut errors);
                    }
                }
                GraphItem::Enum(e) => {
                    for v in &e.variants {
                        match &v.payload {
                            VariantPayload::Unit => {}
                            VariantPayload::Tuple(tys) => {
                                for t in tys {
                                    check_named(t, &defined, &mut errors);
                                }
                            }
                            VariantPayload::Struct(fields) => {
                                for f in fields {
                                    check_named(&f.ty, &defined, &mut errors);
                                }
                            }
                        }
                    }
                }
                GraphItem::Function(f) => {
                    for (_, t) in &f.params {
                        check_named(t, &defined, &mut errors);
                    }
                    check_named(&f.ret, &defined, &mut errors);
                    check_block(&f.body, &struct_fields, &mut errors);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn check_named(ty: &Type, defined: &HashSet<&str>, errors: &mut Vec<String>) {
    match ty {
        Type::Named(n) => {
            if !defined.contains(n.as_str()) {
                errors.push(format!("undefined type: `{n}`"));
            }
        }
        Type::Ref { inner, .. } | Type::Vec(inner) | Type::Option(inner) => {
            check_named(inner, defined, errors)
        }
        Type::Result(a, b) => {
            check_named(a, defined, errors);
            check_named(b, defined, errors);
        }
        Type::Tuple(items) => {
            for t in items {
                check_named(t, defined, errors);
            }
        }
        _ => {}
    }
}

/// Recursively check a block's nodes: required inputs satisfied, plus basic
/// StructLit field typing where a literal is statically known.
fn check_block(
    block: &Block,
    struct_fields: &[(&str, &Vec<(String, Type)>)],
    errors: &mut Vec<String>,
) {
    for (id, node) in &block.nodes {
        check_inputs(block, *id, node, errors);
        if let NodeKind::StructLit { name, fields } = &node.kind {
            if let Some((_, decl)) = struct_fields.iter().find(|(n, _)| n == name) {
                for (i, fname) in fields.iter().enumerate() {
                    if let Some((_, ty)) = decl.iter().find(|(dn, _)| dn == fname) {
                        if let Some(leaf) = inline_of(node, i as u16) {
                            if let Some(msg) = leaf_type_conflict(fname, leaf, ty) {
                                errors.push(msg);
                            }
                        }
                    }
                }
            }
        }
        // Recurse into nested scopes.
        match &node.kind {
            NodeKind::ForEach { body, .. } => check_block(body, struct_fields, errors),
            NodeKind::If { then, els } => {
                check_block(then, struct_fields, errors);
                if let Some(e) = els {
                    check_block(e, struct_fields, errors);
                }
            }
            NodeKind::Match { arms } => {
                for a in arms {
                    check_block(&a.body, struct_fields, errors);
                    if let Some(g) = &a.guard {
                        check_block(g, struct_fields, errors);
                    }
                }
            }
            _ => {}
        }
    }
}

fn inline_of(node: &Node, port: u16) -> Option<&Leaf> {
    node.inline.iter().find(|(p, _)| *p == port).map(|(_, l)| l)
}

fn required_ports(node: &Node) -> Vec<u16> {
    match &node.kind {
        NodeKind::Let { .. } => vec![0],
        NodeKind::Assign { .. } => vec![0, 1],
        NodeKind::ForEach { .. } => vec![0],
        NodeKind::ExprStmt => vec![0],
        NodeKind::Return { has_value } => {
            if *has_value {
                vec![0]
            } else {
                vec![]
            }
        }
        NodeKind::Field { .. } => vec![0],
        NodeKind::Binary { .. } => vec![0, 1],
        NodeKind::Ref { .. } => vec![0],
        NodeKind::Try => vec![0],
        NodeKind::Match { .. } => vec![0],
        NodeKind::If { .. } => vec![0],
        // variadic: at least their fixed prefix
        NodeKind::Call | NodeKind::Method { .. } => vec![0],
        NodeKind::Builtin { .. } => vec![],
        NodeKind::StructLit { fields } => (0..fields.len() as u16).collect(),
    }
}

fn check_inputs(block: &Block, id: NodeId, node: &Node, errors: &mut Vec<String>) {
    for port in required_ports(node) {
        let wired = block.data.iter().any(|e| e.to == id && e.to_port == port);
        let inlined = node.inline.iter().any(|(p, _)| *p == port);
        if !wired && !inlined {
            errors.push(format!("node {id:?} input port {port} is not connected"));
        }
    }
}

/// A statically-detectable literal/field-type conflict, or `None` if compatible
/// or unknowable.
fn leaf_type_conflict(field: &str, leaf: &Leaf, ty: &Type) -> Option<String> {
    let Leaf::Lit(l) = leaf else { return None };
    let ok = match (l, ty) {
        (Literal::Int(_), Type::I32 | Type::I64 | Type::Usize) => true,
        (Literal::Float(_), Type::F64) => true,
        (Literal::Bool(_), Type::Bool) => true,
        (Literal::Char(_), Type::Char) => true,
        (Literal::Str(_), Type::Str | Type::String) => true,
        (Literal::Unit, Type::Unit) => true,
        _ => false,
    };
    if ok {
        None
    } else {
        Some(format!("field `{field}`: literal {l:?} is not compatible with {ty:?}"))
    }
}
```

Add to `crates/graph/src/lib.rs`:

```rust
pub mod validate;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vr-graph validate`
Expected: PASS — all four validation tests green.

- [ ] **Step 5: Commit**

```bash
git add crates/graph
git commit -m "feat(graph): add Graph::validate (names, entry, inputs, basic pin typing)"
```

---

### Task 6: The capstone parity fixture

**Files:**
- Create: `crates/graph/src/fixtures.rs`
- Modify: `crates/graph/src/lib.rs`

**Interfaces:**
- Consumes: `model` + `build` helpers.
- Produces: `pub fn line_report_graph() -> Graph`.

This builds the same program as `vr_ir::fixtures::line_report`, as a graph. Parity is the oracle: the generated source must be byte-identical. Where a construct maps 1:1 (structs, enums, straight-line statements) the graph mirrors the IR; where it differs (match arms, if/else), follow the lowering rules from Task 4.

> Implementer note: build the fixture function-by-function. After writing each
> function's sub-graph, you can spot-check by lowering just that item and
> generating it. The parity test in Step 3 is the real gate; when it fails it
> prints both sources — diff them and fix the fixture (not the generator, and not
> the parity assertion). If parity forces an unnatural graph shape, STOP and ask
> Felipe (per the spec's escape hatch).

- [ ] **Step 1: Write the failing test**

Create `crates/graph/src/fixtures.rs` with the test first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_capstone_matches_ir_capstone_source() {
        let from_graph = crate::lower(&line_report_graph()).expect("graph lowers");
        let graph_src = vr_rustgen::generate(&from_graph).expect("graph generates");
        let ir_src =
            vr_rustgen::generate(&vr_ir::fixtures::line_report()).expect("ir generates");
        assert_eq!(graph_src, ir_src, "graph-derived source must match the IR fixture");
    }

    #[test]
    fn graph_capstone_validates() {
        line_report_graph().validate().expect("capstone graph must validate");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-graph fixtures`
Expected: FAIL — `line_report_graph` not found.

- [ ] **Step 3: Write the fixture**

Prepend to `crates/graph/src/fixtures.rs`. Build with the Task 3 helpers. This mirrors `vr_ir::fixtures::line_report`; consult that file side-by-side.

```rust
use vr_ir::{AssignOp, BinaryOp, BuiltinOp, Literal, Pattern, Type, VariantPayload};

use crate::build::*;
use crate::model::*;

pub fn line_report_graph() -> Graph {
    Graph {
        items: vec![
            report_struct(),
            line_kind_enum(),
            classify_fn(),
            build_report_fn(),
            run_fn(),
            main_fn(),
        ],
    }
}

fn report_struct() -> GraphItem {
    let f = |n: &str| (n.to_string(), Type::Usize);
    GraphItem::Struct(StructDecl {
        name: "Report".into(),
        fields: vec![
            f("total_lines"),
            f("blank_lines"),
            f("comment_lines"),
            f("content_lines"),
            f("words"),
        ],
    })
}

fn line_kind_enum() -> GraphItem {
    let v = |n: &str| VariantDecl { name: n.into(), payload: VariantPayload::Unit };
    GraphItem::Enum(EnumDecl {
        name: "LineKind".into(),
        variants: vec![v("Blank"), v("Comment"), v("Content")],
    })
}

// fn classify(line: &str) -> LineKind {
//   let trimmed = line.trim();
//   if trimmed.is_empty() { LineKind::Blank }
//   else if trimmed.starts_with('#') { LineKind::Comment }
//   else { LineKind::Content }
// }
fn classify_fn() -> GraphItem {
    let mut b = BlockBuilder::new();

    // let trimmed = line.trim();
    let trim = method(&mut b, var("line"), "trim", vec![]);
    let let_trimmed = b.stmt(NodeKind::Let { name: "trimmed".into(), mutable: false });
    b.feed(let_trimmed, 0, trim);

    // inner else block: { LineKind::Content }  -> Expr::Block(tail: Content)
    let mut content_block = BlockBuilder::new();
    let content_path = passthrough_path(&mut content_block, &["LineKind", "Content"]);
    content_block.set_tail(content_path);

    // inner if: if trimmed.starts_with('#') { Comment } else { <content block> }
    let mut inner = BlockBuilder::new();
    let starts = method(
        &mut inner,
        var("trimmed"),
        "starts_with",
        vec![lit(Literal::Char('#'))],
    );
    let comment_then = {
        let mut t = BlockBuilder::new();
        let p = passthrough_path(&mut t, &["LineKind", "Comment"]);
        t.set_tail(p);
        t.build()
    };
    let inner_if = inner.value(NodeKind::If {
        then: comment_then,
        els: Some(content_block.build()),
    });
    inner.feed(inner_if, 0, starts);
    inner.set_tail(inner_if);

    // outer if: if trimmed.is_empty() { Blank } else { <inner if block> }
    let empty = method(&mut b, var("trimmed"), "is_empty", vec![]);
    let blank_then = {
        let mut t = BlockBuilder::new();
        let p = passthrough_path(&mut t, &["LineKind", "Blank"]);
        t.set_tail(p);
        t.build()
    };
    let outer_if = b.value(NodeKind::If { then: blank_then, els: Some(inner.build()) });
    b.feed(outer_if, 0, empty);
    b.set_tail(outer_if);

    GraphItem::Function(FunctionGraph {
        name: "classify".into(),
        params: vec![("line".into(), Type::Str)],
        ret: Type::Named("LineKind".into()),
        body: b.build(),
    })
}

// A path used as a bare tail value needs to be a *node* (tails are node ids).
// Represent `LineKind::Content` as a zero-arg Call-free value: wrap the path
// leaf in a `Field`-free identity by using a Builtin? No — instead model a bare
// path tail as a single `Ref`-free value node. The simplest faithful node is a
// `Call` with the path as func and no args would emit `LineKind::Content()`,
// which is wrong. So introduce a dedicated helper that stores the path on a
// one-output value node via `Method`? Also wrong. The correct representation:
// a path tail is a value; lowering a bare `Leaf::Path` needs a node whose
// `lower_value` yields `Expr::Path`. Add a `NodeKind::PathValue(Vec<String>)`
// is the clean fix — see the note below; implement it.
fn passthrough_path(b: &mut BlockBuilder, segs: &[&str]) -> NodeId {
    b.value(NodeKind::PathValue(segs.iter().map(|s| s.to_string()).collect()))
}
```

> **Model addition discovered while writing the fixture:** a block *tail* is a
> `NodeId` (a value node), but a bare path like `LineKind::Content` is a `Leaf`,
> not a node. Rather than distort the graph, add a small value node kind that
> carries a path. Make these three edits, then continue:
>
> 1. In `model.rs`, add to `NodeKind` (value section):
>    ```rust
>    /// A path used directly as a value, e.g. `LineKind::Blank` as a tail.
>    PathValue(Vec<String>),
>    ```
> 2. In `lower.rs` `lower_value`, add an arm:
>    ```rust
>    NodeKind::PathValue(segs) => Expr::Path(segs.clone()),
>    ```
>    and add `NodeKind::PathValue(_) => vec![]` to `required_ports` in
>    `validate.rs` (no inputs).
> 3. Re-run earlier tests (`cargo test -p vr-graph lower validate`) — still green.
>
> (Var and Lit leaves never appear as bare tails in the capstone, so only
> `PathValue` is needed. If a later fixture needs a bare `Var`/`Lit` tail, add
> `VarValue`/`LitValue` the same way.)

Now continue the fixture with the remaining functions:

```rust
// fn build_report(text: &str) -> Report { ... }
fn build_report_fn() -> GraphItem {
    let mut b = BlockBuilder::new();

    // let mut report = Report { total_lines: 0, ... };
    let init = struct_lit(
        &mut b,
        "Report",
        vec![
            ("total_lines", int(0)),
            ("blank_lines", int(0)),
            ("comment_lines", int(0)),
            ("content_lines", int(0)),
            ("words", int(0)),
        ],
    );
    let let_report = b.stmt(NodeKind::Let { name: "report".into(), mutable: true });
    b.feed(let_report, 0, init);

    // for line in text.lines() { report.total_lines += 1; match classify(line) { ... } }
    let mut body = BlockBuilder::new();

    // report.total_lines += 1;
    bump(&mut body, "total_lines");

    // match classify(line) { Blank => {..}, Comment => {..}, Content => {..} }
    let scrut = call(&mut body, var("classify"), vec![var("line")]);
    let arms = vec![
        arm_bump("Blank", "blank_lines"),
        arm_bump("Comment", "comment_lines"),
        content_arm(),
    ];
    let match_node = body.value(NodeKind::Match { arms });
    body.feed(match_node, 0, scrut);
    let es = body.stmt(NodeKind::ExprStmt);
    body.feed(es, 0, Src::Node(match_node));

    let lines_iter = method(&mut b, var("text"), "lines", vec![]);
    let for_node = b.value(NodeKind::ForEach { binding: "line".into(), body: body.build() });
    // ForEach is a statement; re-insert it on the exec chain. Use stmt() form:
    // (value() above was wrong for a control node — use the stmt path instead.)
    let _ = for_node;
    let for_stmt = b.stmt(NodeKind::ForEach {
        binding: "line".into(),
        body: rebuild_for_body(),
    });
    b.feed(for_stmt, 0, lines_iter);

    b.set_tail_var(&mut 0); // placeholder; replaced below
    // tail: report  -> needs a value node carrying `report`
    let report_tail = b.value(NodeKind::VarValue("report".into()));
    b.set_tail(report_tail);

    GraphItem::Function(FunctionGraph {
        name: "build_report".into(),
        params: vec![("text".into(), Type::Str)],
        ret: Type::Named("Report".into()),
        body: b.build(),
    })
}
```

> **Second model addition (same rationale as `PathValue`):** the function tail
> `report` is a bare `Var`. Add `VarValue(String)` exactly like `PathValue`:
> `NodeKind::VarValue(String)` in `model.rs`; `NodeKind::VarValue(n) =>
> Expr::Var(n.clone())` in `lower.rs`; `NodeKind::VarValue(_) => vec![]` in
> `required_ports`. Delete the `for_node`/`_`/`set_tail_var` scaffolding lines —
> they are illustrative dead-ends; the real control node is `for_stmt` created via
> `b.stmt(...)`, and the body is built inline (see the cleaned version below).
> There is no `set_tail_var` or `rebuild_for_body` to implement; the snippet above
> deliberately shows the wrong turns so you do not leave them in. Use this cleaned
> `build_report_fn` body instead:

```rust
fn build_report_fn() -> GraphItem {
    let mut b = BlockBuilder::new();

    let init = struct_lit(
        &mut b,
        "Report",
        vec![
            ("total_lines", int(0)),
            ("blank_lines", int(0)),
            ("comment_lines", int(0)),
            ("content_lines", int(0)),
            ("words", int(0)),
        ],
    );
    let let_report = b.stmt(NodeKind::Let { name: "report".into(), mutable: true });
    b.feed(let_report, 0, init);

    // loop body
    let mut body = BlockBuilder::new();
    bump(&mut body, "total_lines");
    let scrut = call(&mut body, var("classify"), vec![var("line")]);
    let arms = vec![
        arm_bump("Blank", "blank_lines"),
        arm_bump("Comment", "comment_lines"),
        content_arm(),
    ];
    let match_node = body.value(NodeKind::Match { arms });
    body.feed(match_node, 0, scrut);
    let es = body.stmt(NodeKind::ExprStmt);
    body.feed(es, 0, Src::Node(match_node));

    let lines_iter = method(&mut b, var("text"), "lines", vec![]);
    let for_stmt = b.stmt(NodeKind::ForEach { binding: "line".into(), body: body.build() });
    b.feed(for_stmt, 0, lines_iter);

    let report_tail = b.value(NodeKind::VarValue("report".into()));
    b.set_tail(report_tail);

    GraphItem::Function(FunctionGraph {
        name: "build_report".into(),
        params: vec![("text".into(), Type::Str)],
        ret: Type::Named("Report".into()),
        body: b.build(),
    })
}

/// `report.<field> += 1;` appended as a statement to `b`.
fn bump(b: &mut BlockBuilder, field_name: &str) {
    let target = field(b, var("report"), field_name);
    let asg = b.stmt(NodeKind::Assign { op: AssignOp::Add });
    b.feed(asg, 0, target);
    b.feed(asg, 1, int(1));
}

/// A `LineKind::<variant> => { report.<field> += 1; }` arm.
fn arm_bump(variant: &str, field_name: &str) -> Arm {
    let mut body = BlockBuilder::new();
    bump(&mut body, field_name);
    Arm {
        pattern: Pattern::Path(vec!["LineKind".into(), variant.into()]),
        guard: None,
        body: body.build(),
    }
}

/// `LineKind::Content => { report.content_lines += 1; report.words += line.split_whitespace().count(); }`
fn content_arm() -> Arm {
    let mut body = BlockBuilder::new();
    bump(&mut body, "content_lines");
    let split = method(&mut body, var("line"), "split_whitespace", vec![]);
    let count = method(&mut body, split, "count", vec![]);
    let target = field(&mut body, var("report"), "words");
    let asg = body.stmt(NodeKind::Assign { op: AssignOp::Add });
    body.feed(asg, 0, target);
    body.feed(asg, 1, count);
    Arm {
        pattern: Pattern::Path(vec!["LineKind".into(), "Content".into()]),
        guard: None,
        body: body.build(),
    }
}

// fn run() -> Result<(), String> { ... }
fn run_fn() -> GraphItem {
    let mut b = BlockBuilder::new();

    // let path = match std::env::args().nth(1) {
    //   Some(p) => p,
    //   None => { return Err("usage: report <file>".to_string()); }
    // };
    let nth = builtin(&mut b, BuiltinOp::NthArg(1), vec![]);
    let some_arm = {
        let mut t = BlockBuilder::new();
        let p = t.value(NodeKind::VarValue("p".into()));
        t.set_tail(p);
        Arm {
            pattern: Pattern::TupleStruct {
                path: vec!["Some".into()],
                elems: vec![Pattern::Binding("p".into())],
            },
            guard: None,
            body: t.build(),
        }
    };
    let none_arm = {
        let mut nb = BlockBuilder::new();
        let msg = method(
            &mut nb,
            lit(Literal::Str("usage: report <file>".into())),
            "to_string",
            vec![],
        );
        let err = call(&mut nb, path(&["Err"]), vec![msg]);
        let ret = nb.stmt(NodeKind::Return { has_value: true });
        nb.feed(ret, 0, err);
        Arm {
            pattern: Pattern::Path(vec!["None".into()]),
            guard: None,
            body: nb.build(),
        }
    };
    let match_path = b.value(NodeKind::Match { arms: vec![some_arm, none_arm] });
    b.feed(match_path, 0, nth);
    let let_path = b.stmt(NodeKind::Let { name: "path".into(), mutable: false });
    b.feed(let_path, 0, Src::Node(match_path));

    // let text = read_file(&path)?;
    let read = builtin(
        &mut b,
        BuiltinOp::ReadFileToString,
        vec![reference(&mut b, false, var("path"))],
    );
    let tried = try_(&mut b, read);
    let let_text = b.stmt(NodeKind::Let { name: "text".into(), mutable: false });
    b.feed(let_text, 0, tried);

    // let report = build_report(&text);
    let br = call(
        &mut b,
        var("build_report"),
        vec![reference(&mut b, false, var("text"))],
    );
    let let_report = b.stmt(NodeKind::Let { name: "report".into(), mutable: false });
    b.feed(let_report, 0, br);

    // println!("<label>: {}", report.<field>);
    print_line(&mut b, "lines", "total_lines");
    print_line(&mut b, "blank", "blank_lines");
    print_line(&mut b, "comment", "comment_lines");
    print_line(&mut b, "content", "content_lines");
    print_line(&mut b, "words", "words");

    // tail: Ok(())
    let ok = call(&mut b, path(&["Ok"]), vec![lit(Literal::Unit)]);
    // Ok(()) is a value node (Call) -> can be the tail directly.
    if let Src::Node(id) = ok {
        b.set_tail(id);
    }

    GraphItem::Function(FunctionGraph {
        name: "run".into(),
        params: vec![],
        ret: Type::Result(Box::new(Type::Unit), Box::new(Type::String)),
        body: b.build(),
    })
}

/// `println!("<label>: {}", report.<field>);` appended as a statement.
fn print_line(b: &mut BlockBuilder, label: &str, field_name: &str) {
    let arg = field(b, var("report"), field_name);
    let call_node = builtin(b, BuiltinOp::PrintLine(format!("{label}: {{}}")), vec![arg]);
    let es = b.stmt(NodeKind::ExprStmt);
    b.feed(es, 0, call_node);
}

// fn main() { match run() { Ok(()) => {}, Err(e) => { eprintln!("error: {}", e); exit(1); } } }
fn main_fn() -> GraphItem {
    let mut b = BlockBuilder::new();
    let run_call = call(&mut b, var("run"), vec![]);
    let ok_arm = Arm {
        pattern: Pattern::TupleStruct {
            path: vec!["Ok".into()],
            elems: vec![Pattern::Tuple(vec![])],
        },
        guard: None,
        body: BlockBuilder::new().build(),
    };
    let err_arm = {
        let mut eb = BlockBuilder::new();
        let ep = builtin(&mut eb, BuiltinOp::EPrintLine("error: {}".into()), vec![var("e")]);
        let es = eb.stmt(NodeKind::ExprStmt);
        eb.feed(es, 0, ep);
        let exit = builtin(&mut eb, BuiltinOp::Exit, vec![int(1)]);
        let es2 = eb.stmt(NodeKind::ExprStmt);
        eb.feed(es2, 0, exit);
        Arm {
            pattern: Pattern::TupleStruct {
                path: vec!["Err".into()],
                elems: vec![Pattern::Binding("e".into())],
            },
            guard: None,
            body: eb.build(),
        }
    };
    let match_node = b.value(NodeKind::Match { arms: vec![ok_arm, err_arm] });
    b.feed(match_node, 0, run_call);
    let es = b.stmt(NodeKind::ExprStmt);
    b.feed(es, 0, Src::Node(match_node));

    GraphItem::Function(FunctionGraph {
        name: "main".into(),
        params: vec![],
        ret: Type::Unit,
        body: b.build(),
    })
}
```

Add to `crates/graph/src/lib.rs`:

```rust
pub mod fixtures;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vr-graph fixtures`
Expected: PASS. If the parity assertion fails, it prints both sources — diff and fix the fixture. Common parity pitfalls: a match arm emitted as `{ x }` instead of bare `x` (check `lower_arm_body`), or an `else` emitted bare instead of blocked (check `lower_else`).

- [ ] **Step 5: Run the whole crate suite + gate**

Run:
```bash
cargo test -p vr-graph
cargo fmt -p vr-graph --check
cargo clippy -p vr-graph --all-targets -- -D warnings
```
Expected: all PASS/clean.

- [ ] **Step 6: Commit**

```bash
git add crates/graph
git commit -m "feat(graph): add line-report capstone graph fixture with source parity"
```

---

### Task 7: Generated-code validity from the graph path

**Files:**
- Create: `crates/graph/tests/compile.rs`

**Interfaces:**
- Consumes: `vr_graph::{lower, fixtures}`, `vr_rustgen::generate`.
- Produces: an integration test that compiles + runs the graph-derived program and asserts the Phase 1 counts. Mirrors `crates/rustgen/tests/compile.rs`.

- [ ] **Step 1: Write the test**

Create `crates/graph/tests/compile.rs`:

```rust
//! Generated-code validity from the graph front-end: lower the capstone graph,
//! generate Rust, compile it with the active `rustc`, and run it against a
//! sample input. Std-only, no network. Mirrors the rustgen compile test.

use std::process::Command;

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("vr_graph_{}_{}", std::process::id(), tag));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into())
}

fn generate() -> String {
    let prog = vr_graph::lower(&vr_graph::fixtures::line_report_graph()).unwrap();
    vr_rustgen::generate(&prog).unwrap()
}

#[test]
fn graph_capstone_compiles_with_active_rustc() {
    let src = generate();
    let dir = temp_dir("compile");
    let src_path = dir.join("main.rs");
    std::fs::write(&src_path, &src).unwrap();

    let out = Command::new(rustc())
        .args(["--edition", "2021", "--crate-type", "bin", "--emit=metadata"])
        .arg(&src_path)
        .arg("--out-dir")
        .arg(&dir)
        .output()
        .expect("failed to invoke rustc");

    assert!(
        out.status.success(),
        "generated code failed to compile:\n--- source ---\n{src}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn graph_capstone_runs_and_reports_counts() {
    let src = generate();
    let dir = temp_dir("run");
    let src_path = dir.join("main.rs");
    std::fs::write(&src_path, &src).unwrap();

    let exe = dir.join(if cfg!(windows) { "report.exe" } else { "report" });
    let build = Command::new(rustc())
        .args(["--edition", "2021", "-O"])
        .arg(&src_path)
        .arg("-o")
        .arg(&exe)
        .output()
        .expect("failed to invoke rustc");
    assert!(
        build.status.success(),
        "build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    let input = dir.join("sample.txt");
    std::fs::write(&input, "\n# a comment\nhello world\nthree more words\n").unwrap();

    let run = Command::new(&exe).arg(&input).output().expect("run failed");
    assert!(run.status.success(), "program exited with error: {run:?}");
    let stdout = String::from_utf8_lossy(&run.stdout);

    assert!(stdout.contains("lines: 4"), "got:\n{stdout}");
    assert!(stdout.contains("blank: 1"), "got:\n{stdout}");
    assert!(stdout.contains("comment: 1"), "got:\n{stdout}");
    assert!(stdout.contains("content: 2"), "got:\n{stdout}");
    assert!(stdout.contains("words: 5"), "got:\n{stdout}");
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test -p vr-graph --test compile`
Expected: PASS (both). If parity held in Task 6 this follows, but this proves the graph path independently compiles and runs.

- [ ] **Step 3: Verify the whole workspace is green**

Run:
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
Expected: all PASS/clean.

- [ ] **Step 4: Commit**

```bash
git add crates/graph/tests
git commit -m "test(graph): compile and run graph-derived capstone under active rustc"
```

---

### Task 8: Documentation — ADR and roadmap note

**Files:**
- Create: `docs/adr/0008-vr-graph-headless-front-end.md`
- Modify: `docs/adr/README.md`, `ROADMAP.md`

**Interfaces:**
- Consumes: the completed crate.
- Produces: a recorded decision and an accurate roadmap.

- [ ] **Step 1: Write the ADR**

Create `docs/adr/0008-vr-graph-headless-front-end.md`:

```markdown
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
```

- [ ] **Step 2: Add the index row**

In `docs/adr/README.md`, add a row to the table for `0008` (match the existing table's column format), titled "vr-graph: a headless graph front-end producing IR", Status "Accepted".

- [ ] **Step 3: Note the roadmap progress (still In Progress)**

In `ROADMAP.md`, update the Phase 2 status line to reflect the headless core without marking the phase Done:

```markdown
**Status:** In Progress (headless graph core landed: `vr-graph` model + graph -> IR lowering + validation, proven at capstone parity; Godot `gdext` plugin, canvas, and live panel still pending)
```

Leave the four Phase 2 checkboxes unchecked (the plugin/canvas/panel are not done).

- [ ] **Step 4: Verify the workspace one final time**

Run:
```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```
Expected: all PASS/clean.

- [ ] **Step 5: Commit**

```bash
git add docs/adr ROADMAP.md
git commit -m "docs(graph): add ADR-0008 and note Phase 2 headless-core progress"
```

---

## Self-Review

**1. Spec coverage:**
- "New `vr-graph` crate depending on `vr-ir` only, no syn/quote/godot" -> Task 1 manifest; Global Constraints. Covered.
- "Model: Graph/GraphItem/decls; BodyGraph as nodes+exec+data; Blueprints pins; pragmatic leaves" -> Task 2 (`Block` is the body graph; inline leaves). Covered. (Naming: the spec's `BodyGraph` is implemented as `Block`, uniform across function bodies and nested scopes; noted so a reader is not surprised.)
- "Lowering to vr_ir::Program" -> Task 4. Covered, including the parity-critical bare-arm / else-block / else-if rules.
- "Validation: named types, scope, required inputs, pin type compat (basic)" -> Task 5. Covered; full method/builtin return typing explicitly deferred (ADR-0008, Task 8).
- "Success oracle: byte-identical source parity + compile/run counts" -> Task 6 (parity test) + Task 7 (compile/run). Covered.
- "Deferred: gdext plugin, GraphEdit, live panel; Phase 2 stays In Progress" -> Task 8 (ADR-0008 + roadmap note, boxes left unchecked). Covered.

**2. Placeholder scan:** The fixture task (Task 6) deliberately shows two "wrong turn" snippets (the `passthrough_path` comment discovering `PathValue`, and the `for_node`/`set_tail_var` dead-ends) that are immediately corrected with a cleaned version and explicit edit instructions. These are pedagogical, not placeholders — each is resolved in-task with complete code. The `PathValue`/`VarValue` model additions are fully specified (three concrete edits each). No `TBD`/"add error handling"/"implement later" remain.

**3. Type consistency:** `lower(&Graph) -> Result<vr_ir::Program, LowerError>` used identically in Tasks 4/6/7. `BlockBuilder`, `Src`, and the value helpers (`int`/`var`/`path`/`field`/`method`/`call`/`binary`/`reference`/`struct_lit`/`builtin`/`try_`) defined in Task 3 and used in Tasks 4/5/6. `NodeKind` variants match between `model.rs` (Task 2, plus `PathValue`/`VarValue` added in Task 6), `lower.rs` (Task 4), and `validate.rs` (Task 5). Port conventions are stated once (Task 2 header) and consumed consistently by builders and lowering. IR types (`BuiltinOp`, `Pattern`, `VariantPayload`, `AssignOp`, `BinaryOp`) are reused from `vr_ir`, not redefined.

> One consistency note for the implementer: when you add `PathValue`/`VarValue`
> in Task 6, the `match` in `lower_value` and the `required_ports` in `validate.rs`
> both gain arms; the compiler's exhaustiveness check will flag them if you miss
> one. Let it guide you.
