<p align="center">
  <img src="assets/logo.png" alt="Visual Rust logo: a gear with three wired nodes branching to the right" width="140">
</p>

# Visual Rust

![Status](https://img.shields.io/badge/status-MVP%20in%20progress-yellow)
![Language](https://img.shields.io/badge/language-Rust-orange)
![Editor](https://img.shields.io/badge/editor-Godot%204%20%2B%20gdext-478cbf)
![License](https://img.shields.io/badge/license-TBD-lightgrey)

**Visual Rust** is a node editor for visual programming in Rust. It's a Blueprints-style graph of typed pins and wires that compiles to real, idiomatic Rust source instead of running through an interpreter. Every node graph maps to `.rs` code, built with `cargo build`. What you ship is a native binary: no runtime attached, no scripting layer underneath it.

The interaction model borrows heavily from Unreal Engine Blueprints: typed pins, wires, inline error states. That's the clearest existing reference point for a visual graph that actually reads like code. The signature feature is what it does with ownership. Borrows and moves render as wire state on the canvas, and a borrow-checker violation shows up as an in-graph error, not a wall of compiler output.

## Features

**Working today:**
- Graph → Typed IR → Rust AST pipeline, implemented and CI-tested end to end ([Phase 1](ROADMAP.md))
- Target-agnostic Typed IR — Rust is the primary backend, not a hardcoded one ([ADR-0005](docs/adr/0005-target-agnostic-ir-rust-primary.md))
- Rust code generation through `syn`/`quote`/`prettyplease`, never a hand-rolled string template
- `vr-graph`: a node/pin/wire graph model with validation (names, entry point, inputs, pin typing), lowering directly to the IR
- CI gate on stable Rust (build, test, and a generated-code compile+run check), with non-blocking beta/nightly smoke tests

**On the roadmap** (see [`ROADMAP.md`](ROADMAP.md) for current phase status):
- Godot `GraphEdit`-based visual canvas, built entirely in Rust via `gdext`
- Live, read-only "Generated Rust" panel next to the graph
- Borrow-checker violation visualization directly on wires and pins

## Why

Rust's performance and memory safety come with a steep learning curve, most of it around ownership, lifetimes, and borrowing. Visual Rust gives you a topological view of data flow and ownership, so what the borrow checker enforces becomes something you watch on the canvas instead of something you decode from compiler output. You still ship a real, compiled Rust program.

## Status

Early, active development. See [`ROADMAP.md`](ROADMAP.md) for phase-by-phase status and [`PRD.md`](PRD.md) for the full product spec. Every non-obvious design and architecture decision is recorded in [`docs/adr/`](docs/adr/README.md).

## Documentation

- [`PRD.md`](PRD.md) — vision, architecture, scope
- [`ROADMAP.md`](ROADMAP.md) — phase status, MVP through 1.0
- [`docs/adr/`](docs/adr/README.md) — architecture decision records

## Workspace layout

A Cargo workspace of four crates. `vrc` (in `crates/cli`) runs the pipeline; the rest are libraries.

- `crates/ir` (`vr-ir`) — the target-agnostic Typed IR. No Rust-AST dependencies ([ADR-0005](docs/adr/0005-target-agnostic-ir-rust-primary.md)).
- `crates/graph` (`vr-graph`) — a Blueprints-style node/pin/wire graph model plus lowering to the IR; the headless core of the editor front-end ([ADR-0008](docs/adr/0008-vr-graph-headless-front-end.md)). Depends on `vr-ir` only.
- `crates/rustgen` (`vr-rustgen`) — lowers the IR to Rust source via `syn`/`quote`/`prettyplease`. The only crate that knows Rust syntax.
- `crates/cli` (`vr-cli`) — the `vrc` binary.

Pipeline: `vr-graph` -> `vr-ir` -> `vr-rustgen` -> Rust source. The Godot editor front-end (Phase 2, [ADR-0003](docs/adr/0003-editor-host-platform-godot-gdext.md)) will sit on top of `vr-graph`.

## Rust version policy

- **MSRV:** pinned in the workspace `Cargo.toml` (`workspace.package.rust-version`).
  Keep this line and the manifest in sync when the floor moves.
- **Generated code** targets **edition 2021**. Adopting a new edition requires a
  dedicated ADR first (see [ADR-0007](docs/adr/0007-rust-version-compatibility-policy.md)).
- **CI:** stable is a hard gate (fmt, clippy, build, test, and the generated-code
  compile+run check). Beta and nightly are non-blocking smoke tests that run the
  same suite to catch breakage before it reaches stable. When a new release turns
  a channel red, that is active work until all three channels are green again.
