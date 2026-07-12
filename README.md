# Visual Rust

![Status](https://img.shields.io/badge/status-MVP%20in%20progress-yellow)
![Language](https://img.shields.io/badge/language-Rust-orange)
![Editor](https://img.shields.io/badge/editor-Godot%204%20%2B%20gdext-478cbf)
![License](https://img.shields.io/badge/license-TBD-lightgrey)

**Visual Rust** is a node-based visual programming environment that generates real, idiomatic Rust rather than an interpreted scripting layer. Every node graph maps to `.rs` source, compiled with `cargo build`, so what you build runs as a native binary with no runtime attached.

Its interaction model borrows from Unreal Engine Blueprints: typed pins, wires, and inline error states. The signature feature is making Rust's ownership model visible. Borrows and moves render as wire state on the canvas, and a borrow-checker violation shows up as an in-graph error rather than a wall of compiler text.

## Why

Rust's performance and memory safety come with a steep learning curve, most of it around ownership, lifetimes, and borrowing. Visual Rust gives you a topological view of data flow and ownership, so what the borrow checker enforces becomes something you watch on the canvas instead of something you decode from compiler output. You still ship a real, compiled Rust program.

## Status

Early, active development. See [`ROADMAP.md`](ROADMAP.md) for phase-by-phase status and [`PRD.md`](PRD.md) for the full product spec. Every non-obvious design and architecture decision is recorded in [`docs/adr/`](docs/adr/README.md).

## Documentation

- [`PRD.md`](PRD.md) — vision, architecture, scope
- [`ROADMAP.md`](ROADMAP.md) — phase status, MVP through 1.0
- [`docs/adr/`](docs/adr/README.md) — architecture decision records
