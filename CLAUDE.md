# Visual Rust — project conventions

These are the repo-local conventions distilled from global preferences. They
apply to everyone (human or agent) working in this repository.

## Commits and PRs
- Use **Conventional Commits** (`feat:`, `fix:`, `docs:`, `test:`, `chore:`,
  `refactor:`, `ci:`), scoped where useful (`feat(ir): ...`).
- Commit per completed task; push as work progresses.
- **Never open a pull request unless explicitly asked** in that same request.
- **No AI attribution** anywhere: no `Co-Authored-By` trailers, no "Generated
  with ..." lines in commits, PRs, code, or docs.

## Writing
- **No emojis** anywhere: code, comments, docs, commit messages, chat.
- **Outward-facing non-technical prose** (README, announcements, marketing copy)
  must go through the humanizer pass before publishing, to strip AI-writing tells
  (em-dash-as-aside, "not just X, but Y" parallelism, uniform sentence rhythm,
  repeated stock adjectives). Technical documents are exempt: ADRs, PRD, code
  comments, this file.
- Keep the doc structure: `PRD.md` (stable vision), `ROADMAP.md` (phase status),
  `docs/adr/` (one MADR-lite file per decision, immutable once Accepted; supersede
  via a new ADR). Decisions are settled in the ADRs. Do not silently diverge; if
  something seems wrong, stop and ask.

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
- MVP scope is CLI/scripting only (ADR-0002): no async, GUI, servers, or custom
  trait/generic authoring.
- Emit Rust through `syn`/`quote`/`proc-macro2`, never a hand-rolled AST/string layer.
