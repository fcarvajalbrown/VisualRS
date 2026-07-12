# Visual Rust

![Status](https://img.shields.io/badge/status-MVP%20in%20progress-yellow)
![Language](https://img.shields.io/badge/language-Rust-orange)
![Editor](https://img.shields.io/badge/editor-Godot%204%20%2B%20gdext-478cbf)
![License](https://img.shields.io/badge/license-TBD-lightgrey)

**Visual Rust** is a node-based visual programming environment that generates real, idiomatic Rust — not an interpreted scripting layer. Every node graph maps directly to `.rs` source compiled with `cargo build`, so what you build runs as a native binary with zero added runtime.

Its interaction model is inspired by Unreal Engine Blueprints: typed pins, wires, and inline error states. Its signature feature is making Rust's ownership model visible — borrows and moves render as wire state on the canvas, and borrow-checker violations surface as an in-graph error instead of a wall of compiler text.

## Why

Rust's performance and memory safety come at the cost of a steep learning curve, largely around ownership, lifetimes, and borrowing. Visual Rust gives you a topological view of data flow and ownership so you can see — not just read — what the borrow checker is enforcing, while still shipping a real, compiled, idiomatic Rust program.

## Status

Early, active development. See [`ROADMAP.md`](ROADMAP.md) for phase-by-phase status and [`PRD.md`](PRD.md) for the full product spec. Every non-obvious design and architecture decision is recorded in [`docs/adr/`](docs/adr/README.md).

## Documentation

- [`PRD.md`](PRD.md) — vision, architecture, scope
- [`ROADMAP.md`](ROADMAP.md) — phase status, MVP through 1.0
- [`docs/adr/`](docs/adr/README.md) — architecture decision records
