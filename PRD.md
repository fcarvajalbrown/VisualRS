# PRD: Visual Rust

> Architectural and UX decisions referenced below are tracked in [`docs/adr/`](docs/adr/README.md). Phase status and what's actively being worked on lives in [`ROADMAP.md`](ROADMAP.md). This document is the stable "what and why" — it should change rarely; decisions and schedule churn belong in those other two files.

## Vision Statement

Visual Rust is a visual authoring environment for building high-performance, memory-safe applications. It is not a replacement for Rust, but a visual compiler front-end where every node graph maps directly to idiomatic Rust code. Its interaction model is directly inspired by Unreal Engine Blueprints — wires, typed pins, and inline error states — because that's the closest existing reference point for "a visual graph that reads like real code."

## Problem Statement

Rust offers unmatched performance and safety, but its steep learning curve — particularly around ownership, lifetimes, and borrowing — creates a high barrier to entry. Visual Rust provides an intuitive, topological interface to abstract syntax, allowing developers to see the flow of data, memory ownership, and execution paths in real time.

## Target Users

- Students and educators transitioning into systems programming.
- Automation engineers building tooling without wanting to write boilerplate.
- Indie developers and technical teams prototyping native desktop applications.
- Rust developers seeking a visual overview of complex architectural modules.

## Non-Goals

- Not a general-purpose Blueprint-style scripting *runtime* — there is no interpreter shipped with generated programs. Output is always compiled, native Rust.
- Not targeting mobile or WASM output in the MVP or 1.0 (revisit post-1.0).
- Not a Godot competitor — Godot is a build-time implementation detail of the editor (see [ADR-0003](docs/adr/0003-editor-host-platform-godot-gdext.md)), not a runtime dependency of programs you build with Visual Rust.
- Not attempting full language coverage on day one — MVP scope is deliberately narrow (see [ADR-0002](docs/adr/0002-mvp-scope-cli-scripting-only.md)).

## Core Principles

- **Graphs Are Code:** The visual topology must have a strict, deterministic mapping to Rust constructs.
- **Rust Is the Primary Target, Not the Only One:** The Typed IR is designed target-agnostic from day one — graph → IR → backend, not graph → IR → Rust-only — so a GDScript or C# backend is architecturally possible post-1.0. Rust is the sole backend actually shipped through 1.0 ([ADR-0005](docs/adr/0005-target-agnostic-ir-rust-primary.md)).
- **Zero Interpreted Overhead:** Generated programs compile via `cargo build` to native executables with no proprietary runtime attached. This holds regardless of what the editor itself is built on.
- **Transparent Generation:** Generated Rust code is always visible, readable, and well-structured, never obfuscated.
- **Beginner-Friendly, Expert-Capable:** Abstractions handle basic layouts, while advanced users can drop down into custom Rust nodes.

## System Architecture

**The Pipeline:**

1. **Editor Shell:** A Godot Engine editor plugin, written entirely in Rust via `gdext` (no GDScript). The infinite canvas uses Godot's built-in `GraphEdit`/`GraphNode` widgets rather than a hand-built canvas. See [ADR-0003](docs/adr/0003-editor-host-platform-godot-gdext.md) for why.
2. **Semantic Analyzer:** Validates node connections against Rust's type system and ownership rules before code generation.
3. **Typed Intermediate Representation (IR):** An abstract, graph-agnostic *and* target-agnostic format bridging the visual layout and the code generator ([ADR-0005](docs/adr/0005-target-agnostic-ir-rust-primary.md)).
4. **Rust AST Generator:** Translates the IR into a valid `syn`/`quote`-based Rust Abstract Syntax Tree. The only backend implemented through 1.0.
5. **Cargo Orchestrator:** Shells out to `cargo check`, `cargo build`, and `cargo run`, parsing structured JSON diagnostics (`cargo check --message-format json`) back into the editor for per-node/per-wire error highlighting — not raw terminal text.

## Language Features & Visual Mapping

- **Standard Features:** Functions, structs, enums, pattern matching, `Result`/`Option` handling.
- **Visualizing Ownership (Crucial UX):** Wires are pre-differentiated by weight/color (thin blue = immutable borrow, thick orange = mutable borrow). Invalid borrows flash the offending wire red with a no-entry badge at the violating pin and a plain-language tooltip. See [ADR-0001](docs/adr/0001-borrow-violation-visualization.md) for the full decision and rationale.
- **Error Bubbling:** Rust compiler errors are parsed from structured JSON and mapped back to the specific offending nodes/wires on the graph, avoiding cryptic terminal output.
- **Output GUI Apps (1.0):** Generated GUI programs target `egui`, chosen for its 1:1 immediate-mode-to-node-graph codegen simplicity and permissive licensing, implemented behind a swappable GUI-backend abstraction so alternative toolkits (e.g. Slint) can be added later without redesigning the IR. See [ADR-0006](docs/adr/0006-output-gui-toolkit-egui-swappable.md).

## Prior Art / Building Blocks

Not building from scratch where mature building blocks exist:

- **`syn` + `quote` + `proc-macro2`** — the standard Rust AST parse/generate toolchain; this is the Rust AST Generator stage.
- **`cargo_metadata`** — parses `cargo check --message-format json` output for the Error Bubbling feature.
- **Godot `GraphEdit`/`GraphNode`** — built-in node-graph UI widgets, replacing a hand-built canvas.
- **`godot-rust`/`gdext`** — mature (2026) Rust bindings for Godot 4, used for real editor plugins and tools, not just games.
- **RustViz** (`rustviz/rustviz`) and **BorIs** (`ChristianSchott/boris`) — existing academic/community ownership-visualization tools, worth studying for visual vocabulary before implementing ADR-0001.

**Cautionary references:**
- **Godot's own VisualScript was removed in 4.0** — stated reasons: no high-level domain components bundled with it, and poor documentation. Direct lesson: a bare node-to-Rust mapper without a real standard library of useful nodes is not optional polish — it's what makes the tool useful at all. This is why Phase 4 (Standard Library Nodes) is treated as core MVP scope, not a stretch goal.
- **NetPrints** (.NET, UE-Blueprints-inspired, compiles to C#) is the closest existing analog to this project for a different language — worth studying what it got right and wrong.

## Success Criteria

- **Functional:** A user can open Visual Rust, place a web request node, parse the JSON response, print it to the console, and compile the native executable without typing a single line of Rust.
- **Performance:** The visual editor maintains 60 FPS while navigating a graph with over 500 nodes.
- **Usability:** A developer with basic Python or JavaScript experience can successfully build and run a file-parsing utility within their first hour of using the tool.
