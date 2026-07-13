# Phase 2 (headless core): the `vr-graph` model and graph -> IR lowering

**Status:** Approved (design)
**Date:** 2026-07-12
**Deciders:** Felipe Carvajal Brown

## Context and scope

Phase 2 in `ROADMAP.md` is "Editor Shell & Canvas": a Godot editor plugin in
Rust via `gdext`, using `GraphEdit`/`GraphNode`, with visual type-checking and a
live "Generated Rust" panel. Godot is not installed in the current environment,
and the interactive canvas needs a human validating it in the Godot GUI, so it
cannot be built-and-verified headlessly.

This spec covers only the **headless core** of Phase 2: the pure-Rust pieces that
can be built and unit-tested now, and that the eventual plugin will sit on top
of. Concretely, a new `vr-graph` crate providing:

1. A graph **model** (Blueprints-style nodes, typed pins, exec/data wires).
2. **Lowering** from that model to a `vr_ir::Program`.
3. Graph-level **type-checking / validation** (reject invalid wires before lowering).

The Godot `gdext` plugin, `GraphEdit` wiring, the live source panel, and
interactive validation are explicitly **deferred** to a follow-up (they need the
engine installed and manual GUI validation, and will get their own ADR).
Roadmap Phase 2 therefore stays **In Progress** after this work, not Done.

The interaction model is anchored by the PRD and README to **Unreal
Blueprints-style pins** (typed data pins plus white exec-flow pins) and by
[ADR-0003](../../adr/0003-editor-host-platform-godot-gdext.md) to Godot's
`GraphEdit`. The model below is designed so it maps onto that canvas with minimal
rework.

## Success oracle

The proof that the core works is **full capstone parity** against Phase 1:

- `vr_graph::fixtures::line_report_graph()` -> `lower()` -> a `vr_ir::Program`.
- **Parity:** `vr_rustgen::generate(&lowered)` equals
  `vr_rustgen::generate(&vr_ir::fixtures::line_report())` (byte-identical source).
- **Compile + run:** a `tests/compile.rs` (mirroring Phase 1's Task 12) compiles
  the graph-derived program with the active `rustc` and runs it against the
  sample input, asserting the same counts: `lines: 4, blank: 1, comment: 1,
  content: 2, words: 5`.

Escape hatch: if exact source parity turns out to force an *unnatural* graph
shape (e.g. contorting the model just to match how a nested expression is
grouped), stop and ask Felipe rather than distorting the model. Parity is the
oracle, not a reason to make the graph lie about what a user would build.

## Architecture and layering

New workspace member `crates/graph`, lib name `vr_graph`.

- **Depends on `vr-ir` only.** No `syn`/`quote`/`proc-macro2`, no `godot`/`gdext`.
- `vr-graph` is a **front-end**: it produces a `vr_ir::Program` and hands off to
  the existing backend unchanged. This keeps the ADR-0005 layering intact:

  ```
  vr-graph  ->  vr-ir  ->  vr-rustgen  ->  Rust source
  (front-end)   (IR)       (backend)
  ```

- The deferred plugin will wrap this crate, not replace it:

  ```
  Godot GraphEdit  <->  vr_graph::Graph  ->  lower()  ->  vr_ir  ->  vr_rustgen
  (UI, later)           (model, now)         (now)        (done)     (done)
  ```

- edition 2021, MSRV 1.94 (workspace-inherited), same fmt/clippy/test gates.

## Model

`vr_ir::Type` is reused as the shared type vocabulary. The graph does not define
its own parallel type enum.

Top level:

- `Graph { items: Vec<GraphItem> }`
- `GraphItem = Struct(StructDecl) | Enum(EnumDecl) | Function(FunctionGraph)`
  - `StructDecl { name: String, fields: Vec<(String, Type)> }`
  - `EnumDecl { name: String, variants: Vec<(String, VariantPayload)> }` (payload
    mirrors the IR: unit / tuple / struct)
  - `FunctionGraph { name: String, params: Vec<(String, Type)>, ret: Type, body: BodyGraph }`

Structs and enums are declarative forms, not wired canvases: you fill in fields
and variants, you do not wire a struct definition. Only function bodies are
node graphs.

Function body:

- `BodyGraph { nodes: BTreeMap<NodeId, Node>, exec_edges: Vec<ExecEdge>, data_edges: Vec<DataEdge>, entry: NodeId }`
- `NodeId(u32)` newtype; stable map keys for lookup and edge endpoints.
- `ExecEdge { from: (NodeId, ExecOut), to: NodeId }` threads statement/control
  nodes in execution order. `ExecOut` names which exec output (e.g. a `ForEach`
  has `Body` and `After`; an `If`/`Match` has one per branch/arm).
- `DataEdge { from: (NodeId, OutPort), to: (NodeId, InPort) }` wires a value
  node's typed output pin into another node's typed input pin.

Node kinds:

- **Exec / control** (have exec pins): `Entry`, `Let { name, mutable }`,
  `Assign { op }`, `ForEach { binding }`, `Return`, `ExprStmt`. `If` and `Match`
  appear here (statement position) and also as value nodes (expression position,
  e.g. `classify`'s if/else-if/else tail).
- **Value** (pure, no exec pins): `Var(String)`, `Path(Vec<String>)`,
  `FieldAccess { name }`, `MethodCall { method }`, `Call`, `Binary { op }`,
  `Ref { mutable }`, `StructLit { name, field_names }`, `Builtin { op }`, `Try`.

**Pragmatic leaves.** A data-input pin is satisfied by *either* a wire from a
value node's output pin, *or* an inline leaf payload for trivial atoms. Inline
leaves cover integer/char/string literals, format-string templates (`"lines: {}"`),
and path constants (`LineKind::Blank`). The structurally meaningful data flow is
wired (e.g. the `report` binding feeding field-assignment targets, `line` feeding
method calls); trivial constants are not decomposed pin-by-pin. This keeps the
fixture the size a person would actually build while staying a real graph.

## Lowering

`pub fn lower(graph: &Graph) -> Result<vr_ir::Program, LowerError>`

- Map `StructDecl`/`EnumDecl` straight to `vr_ir::StructDef`/`EnumDef`.
- For each `FunctionGraph`: start at `entry`, follow `exec_edges` to emit a
  `Vec<Stmt>` in order plus an optional tail expression; for each node, resolve
  its data-input pins to `Expr`s (wire -> recurse into the source value node;
  inline -> build the leaf `Expr`). Control nodes recurse into their exec
  sub-flows (`ForEach` body, `If`/`Match` branches) to build nested `Block`s.
- Result is an ordinary `vr_ir::Program`; `vr_rustgen::generate` consumes it
  unchanged.
- `LowerError` reports internal inconsistencies that validation is expected to
  have already caught (defensive; a valid graph should never hit them).

## Type-checking / validation

`impl Graph { pub fn validate(&self) -> Result<(), Vec<String>> }`

Graph-level well-formedness, run before lowering, collecting all problems:

- Exec flow: `entry` exists and is reachable; every control node's exec outputs
  go somewhere valid; no exec cycles except through `ForEach` bodies.
- Data pins: every required input pin is satisfied (a wire or an inline leaf);
  no input pin has two sources.
- **Pin type compatibility:** a `DataEdge`'s source output type matches the
  destination input's expected type (the "reject invalid wire at the pin" rule
  from the roadmap). Exactness first; `Ref` nodes make borrows explicit rather
  than being inferred.
- Names/scope: referenced `Var` bindings are in scope at that point; every
  `Type::Named(n)` refers to a declared struct/enum (as in
  `vr_ir::Program::validate`).

Lowering assumes a validated graph. The deferred plugin will call `validate` live
to drive per-pin error rendering.

## Testing

- `vr_graph::fixtures::line_report_graph()` — the hand-built parity fixture.
- Unit tests: parity (generated source equals the Phase 1 fixture's), and
  `validate` cases (type-mismatched pin rejected, out-of-scope var rejected,
  undefined named type rejected, well-formed graph passes).
- `crates/graph/tests/compile.rs` — compile + run the graph-derived program under
  the active `rustc`, asserting the same counts as Phase 1. Under CI's
  stable/beta/nightly matrix this extends the ADR-0007 validity check to the
  graph path.

## Out of scope (deferred)

- The `gdext` editor-plugin crate and any Godot dependency.
- `GraphEdit`/`GraphNode` wiring, node placement/drag/connect UI.
- The live "Generated Rust" read-only panel.
- Interactive, on-canvas validation rendering (wire colors, per-pin badges).
- `cargo check` JSON diagnostic mapping (that is Phase 3).

These land in a follow-up once Godot is installed, with their own ADR note, and
flip Roadmap Phase 2 to Done at that point.
