# Roadmap

No calendar dates. Phase order is fixed; timing isn't — this is a solo passion project, so each phase carries a status you update as work happens, not a month number. Started 2026-07-12.

Status values: `Not Started` / `In Progress` / `Blocked` / `Done`.

## MVP — CLI/Scripting Only

Scope locked by [ADR-0002](docs/adr/0002-mvp-scope-cli-scripting-only.md): structs, enums, functions, pattern matching, `Result`/`Option`, file I/O, string/collection processing, one-shot sync HTTP calls. No async, no GUI, no networking servers, no custom trait/generic authoring.

### Phase 1: Foundation & Pipeline
**Status:** Done (completed 2026-07-12)
- [x] Define the Typed IR specification (target-agnostic per [ADR-0005](docs/adr/0005-target-agnostic-ir-rust-primary.md))
- [x] Build the core translation engine: hardcoded graph data → IR → Rust AST
- [x] Establish CI: test generated-code validity against stable Rust; smoke-test against beta/nightly per [ADR-0007](docs/adr/0007-rust-version-compatibility-policy.md)

### Phase 2: Editor Shell & Canvas
**Status:** In Progress (headless graph core landed: `vr-graph` model + graph -> IR lowering + validation, proven at capstone parity. The standalone `egui`/`eframe` editor **walking skeleton** now renders the seed graph read-only on an `egui_snarl` canvas — Blueprint-style entry node plus execution/data pins and wires laid out as a begin-to-end flow — with a live read-only "Generated Rust" panel. Canvas editing, pin type-checking, and live-from-canvas regeneration are still pending — see [ADR-0008](docs/adr/0008-vr-graph-headless-front-end.md), [ADR-0009](docs/adr/0009-editor-host-standalone-egui-app.md), [ADR-0010](docs/adr/0010-node-graph-crate-egui-snarl.md), and the walking-skeleton spec in `docs/superpowers/specs/`.)

The editor is a standalone native cross-platform desktop app (a real `.exe` on Windows, plus macOS/Linux) built in Rust with `egui`, with no engine to install ([ADR-0009](docs/adr/0009-editor-host-standalone-egui-app.md), superseding the earlier Godot-plugin host in [ADR-0003](docs/adr/0003-editor-host-platform-godot-gdext.md)). It is built on the headless `vr-graph` core: the canvas adapts its connection state into a `vr_graph::Graph`, then calls `validate()` and `lower()`.

- [x] Stand up the standalone `egui`/`eframe` editor app skeleton (`vr-editor` crate), producing a native binary on Windows/macOS/Linux ([ADR-0009](docs/adr/0009-editor-host-standalone-egui-app.md))
- [ ] Wire up the `egui` node canvas for node placement, dragging, connections
- [ ] Basic visual type-checking (reject invalid wire connections at the pin), driven by `vr_graph::Graph::validate()`
- [ ] Live "Generated Rust" read-only panel (via `vr-graph` `lower()` + `vr-rustgen`)

### Phase 3: Ownership Mechanics
**Status:** Not Started (design: [ADR-0001](docs/adr/0001-borrow-violation-visualization.md))
- [ ] Semantic validation for the borrow checker in the visual space
- [ ] Wire-state rendering: thin blue (immutable) / thick orange (mutable) / red+badge (conflict)
- [ ] `cargo check --message-format json` parsing → per-node/per-wire error mapping
- [ ] Enable graph compilation linking to a local Cargo installation

### Phase 4: Standard Library Nodes
**Status:** Not Started
- [ ] Core node library: primitives, basic math, string manipulation, standard control flow
- [ ] File I/O nodes
- [ ] Reliable cross-platform compilation targets (Windows and Linux)
- Not optional polish — see PRD "Prior Art / Building Blocks" cautionary note on why Godot's VisualScript failed without this.

### Phase 5: MVP Polish & Release
**Status:** Not Started
- [ ] End-to-end bug fixing and UI polish
- [ ] Beginner tutorials and sample CLI utilities
- [ ] Beta release to a small group of technical testers

### Phase 5b: Guided Tutorials & Learning Track
**Status:** Not Started (scope TBD — pending a dedicated brainstorm with Felipe)
- [ ] Scope this out in depth (interactive in-editor learning path vs. written docs vs. sample-driven walkthroughs — decide via the brainstorming skill)
- Placeholder: the PRD names students/educators as core users; this phase is where the in-depth teaching material lives, beyond Phase 5's "beginner tutorials" bullet.

## Post-MVP → 1.0

Scope: [ADR-0005](docs/adr/0005-target-agnostic-ir-rust-primary.md) (Rust stays sole backend) + async + GUI apps + Plugin SDK.

### Phase 6: Plugin SDK
**Status:** Not Started
- [ ] Design the Plugin SDK: wrap external crates as visual nodes via macro attributes or metadata files
- [ ] Flagship domain pack: Godot GDExtension via `gdext` — visual nodes generate real Rust code implementing Godot `Node` classes, running natively inside Godot ([ADR-0004](docs/adr/0004-flagship-domain-godot-gdext-gamedev.md))

### Phase 7: Async/Concurrency Support
**Status:** Not Started
- [ ] async/await visual mapping
- [ ] Task/executor model nodes

### Phase 8: GUI Output
**Status:** Not Started
- [ ] `egui` backend implementation behind the swappable GUI-backend trait ([ADR-0006](docs/adr/0006-output-gui-toolkit-egui-swappable.md))
- [ ] Core widget node set (window, button, text input, layout containers)

### Phase 9: 1.0 Polish & Launch
**Status:** Not Started
- [ ] End-to-end bug fixing across MVP + Post-MVP feature set
- [ ] Full documentation pass, three sample applications
- [ ] 1.0 public release
