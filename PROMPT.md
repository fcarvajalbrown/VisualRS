You are continuing the Visual Rust project (a visual-programming-to-Rust codegen tool).
Working directory: C:\Projects\VisualRS (Windows, PowerShell primary, Bash tool available;
rustc/cargo 1.94.0 stable installed, plus nightly toolchains). Git remote origin =
https://github.com/fcarvajalbrown/VisualRS (public). Everything below is already on `main`
(committed + pushed); there is no open feature branch.

READ FIRST, in this order, before doing anything else:
1. CLAUDE.md (repo root) — project conventions. Obey them exactly.
2. PRD.md, ROADMAP.md, and docs/adr/*.md — the settled product spec and architecture
   decisions. Do NOT re-derive or second-guess them. If something seems wrong or you need to
   deviate, STOP and ask the user (via the interactive option UI) rather than silently
   diverging.
3. docs/superpowers/specs/2026-07-12-phase2-headless-graph-core-design.md and
   docs/superpowers/plans/2026-07-12-phase2-headless-graph-core.md — the Phase 2 headless
   design + plan that produced the current vr-graph crate.
4. docs/adr/0008-vr-graph-headless-front-end.md — the layering decision for vr-graph.

WHAT IS ALREADY DONE (all merged to main; green on fmt + clippy -D warnings + test):
- Cargo workspace, 4 crates. MSRV 1.94 (root Cargo.toml). Generated code targets edition 2021.
- crates/ir (vr-ir): the target-agnostic Typed IR. ZERO syn/quote/proc-macro2 dependency and
  MUST stay that way (ADR-0005).
- crates/rustgen (vr-rustgen): lowers IR -> readable Rust via syn/quote/proc-macro2 +
  prettyplease. The ONLY crate that knows Rust syntax. tests/compile.rs compiles + runs the
  generated capstone (ADR-0007 validity check).
- crates/cli (vr-cli): the `vrc` binary; emits the generated capstone to stdout or --out <path>.
- crates/graph (vr-graph): Phase 2 HEADLESS CORE. A Blueprints-style node/pin/wire model
  (model.rs), lower() to vr_ir (lower.rs), Graph::validate() (validate.rs), a BlockBuilder
  (build.rs), and fixtures::line_report_graph(). Depends on vr-ir ONLY — no syn/quote, no
  godot/gdext. Proven at byte-identical source parity with vr_ir::fixtures::line_report, and
  the graph-derived program compiles + runs (tests/compile.rs). Pipeline:
  vr-graph -> vr-ir -> vr-rustgen.
- CI: .github/workflows/ci.yml (stable hard gate: fmt + clippy + build + test; beta + nightly
  smoke via continue-on-error).

WHAT IS NEXT: finish Phase 2 — the Godot editor plugin (the deferred half).
- Per ADR-0003 the editor is a Godot 4 editor plugin written entirely in Rust via gdext, using
  the built-in GraphEdit/GraphNode widgets, with a live "Generated Rust" panel and on-canvas
  type-check rendering (roadmap Phase 2's four bullets).
- This needs Godot 4 + gdext INSTALLED — they are NOT installed in this environment. Confirm
  the toolchain with the user before planning implementation.
- Build ON TOP of vr-graph: a new plugin crate adapts GraphEdit's connection state into a
  vr_graph::Graph, then calls Graph::validate() (to drive per-pin error rendering) and lower()
  (to feed vr-rustgen for the live panel). Do NOT reimplement the model or lowering in the
  plugin.
- Process (per CLAUDE.md + the user's global prefs): brainstorm scope with the user first
  (superpowers:brainstorming), write a new ADR for the plugin, then a TDD implementation plan
  (superpowers:writing-plans), then execute (superpowers:executing-plans). Flip ROADMAP Phase 2
  to Done only when the plugin ships.
- Full pin-type inference (method/builtin return types) is deferred to Phase 4 (Standard
  Library Nodes), which supplies the type table it needs. vr-graph's validate() currently
  type-checks only statically-knowable conflicts.

HARD RULES (from CLAUDE.md and the user's global preferences — these override defaults):
- No emojis anywhere. No AI attribution anywhere (no Co-Authored-By, no "Generated with ..."
  lines) in commits, PRs, code, or docs.
- Conventional Commits (feat:/fix:/docs:/test:/ci:/chore:/refactor:), scoped where useful.
  Commit per task; push as work progresses. Work on a feature branch; do NOT merge to main or
  open a PR unless the user explicitly asks in that same turn.
- Present every decision via the interactive option UI (one option marked "(Recommended)",
  placed first, with the reasoning stated). If in doubt about anything, ask first — never assume.
- Layering: vr-ir stays target-agnostic (never syn/quote/proc-macro2 or godot). vr-graph is a
  front-end producing IR (vr-ir only, never godot/syn). Only vr-rustgen knows Rust syntax. The
  Godot/gdext dependency belongs ONLY to the future editor-plugin crate.
- Outward-facing non-technical prose (e.g. README marketing copy) goes through the humanizer
  skill before publishing; technical docs (ADRs, PRD, specs, plans, code, CLAUDE.md) are exempt.
- Before committing a crate: cargo fmt --check and cargo clippy --all-targets -- -D warnings
  (CI enforces both). Never claim a task passes without running it.
