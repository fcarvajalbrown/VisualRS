# Visual Rust — project conventions

These are the repo-local conventions distilled from global preferences. They
apply to everyone (human or agent) working in this repository.

## Commits and PRs
- Use **Conventional Commits** (`feat:`, `fix:`, `docs:`, `test:`, `chore:`,
  `refactor:`, `ci:`), scoped where useful (`feat(ir): ...`).
- Commit per completed task, locally, as work progresses — use Conventional
  Commits and split work into logical commits as you go. This is local-only.
- **NEVER push to `origin` (any branch, and especially `origin/main`) without
  Felipe's explicit permission in that same turn.** Committing locally is always
  fine and expected; publishing to the remote is not. Do not push a feature
  branch, do not push `main`, do not `git push` at all until Felipe says to.
  Wait for the go-ahead every time — a prior push being allowed does not authorize
  the next one.
- **Never open a pull request unless explicitly asked** in that same request.
- **No AI attribution** anywhere: no `Co-Authored-By` trailers, no "Generated
  with ..." lines in commits, PRs, code, or docs.

## Writing
- **No emojis** anywhere: code, comments, docs, commit messages, chat.
- **Outward-facing non-technical prose** (README, GitHub repo description/topics,
  announcements, marketing copy) must go through the humanizer pass before
  publishing, to strip AI-writing tells (em-dash-as-aside, "not just X, but Y"
  parallelism, uniform sentence rhythm, repeated stock adjectives). Technical
  documents are exempt: ADRs, PRD, code comments, this file.
- Keep the doc structure: `PRD.md` (stable vision), `ROADMAP.md` (phase status),
  `docs/adr/` (one MADR-lite file per decision, immutable once Accepted; supersede
  via a new ADR). Decisions are settled in the ADRs. Do not silently diverge; if
  something seems wrong, stop and ask.
- **An ADR is never rewritten, extended, or added to.** A new or changed decision
  ALWAYS gets its own new, next-numbered ADR — never a new section bolted onto an
  existing one, never an edit to an Accepted ADR's body. The only edit ever made to
  an existing ADR is flipping its `Status` line to `Superseded by 00YY`. Do not even
  propose folding a decision into an existing ADR.

## Rust version policy (see docs/adr/0007)
- Generated code targets **edition 2021**. Adopting a new edition needs a
  dedicated ADR first.
- CI validates generated code on **stable** (hard gate) and smoke-tests **beta**
  and **nightly** (non-blocking).
- When a new Rust release lands and a channel goes red, treat it as active work:
  stay on it until stable, beta, and nightly are all green again.
- MSRV is pinned in the workspace `Cargo.toml` (`workspace.package.rust-version`)
  and documented in `README.md`.

## Architecture guardrails
- `vr-ir` is target-agnostic (ADR-0005): it must never depend on `syn`/`quote`/
  `proc-macro2` or leak Rust-AST types. Only `vr-rustgen` knows Rust syntax.
- `vr-graph` is a front-end that lowers a graph model to `vr-ir` (ADR-0008): it
  depends on `vr-ir` only — never `syn`/`quote`, `egui`, or `godot`/`gdext`.
  Pipeline: `vr-graph` -> `vr-ir` -> `vr-rustgen`.
- The editor is a standalone native app built with `egui`/`eframe` (ADR-0009,
  superseding the Godot-plugin host of ADR-0003), living in the `vr-editor` crate.
  `vr-editor` depends on `egui`/`eframe` + an `egui` node-graph crate + `vr-graph`
  + `vr-rustgen`; it must not depend on `syn`/`quote`/`proc-macro2` directly.
  `godot`/`gdext` is not used anywhere in the MVP — it belongs only to the future
  post-1.0 Godot GDExtension output pack (ADR-0004), never the editor.
- MVP scope is CLI/scripting only (ADR-0002): no async, GUI, servers, or custom
  trait/generic authoring.
- Emit Rust through `syn`/`quote`/`proc-macro2`, never a hand-rolled AST/string layer.
- The node-graph canvas crate is **`egui_snarl`** (ADR-0010); it was picked over
  `egui_node_graph2` on a maintenance/version check.

## Dependencies (see ADR-0011)
- **Upstream improvements, don't fork.** When our work produces a fix or
  improvement to any third-party dependency (e.g. `egui_snarl`, `egui`/`eframe`),
  contribute it back upstream as a pull request rather than carrying a private
  patch or fork. A vendored patch is a last resort for an unmerged-but-needed
  change; document it and track it for removal once upstream ships.
