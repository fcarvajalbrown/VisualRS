# 0002 — MVP scope: CLI/scripting only

- **Status:** Accepted
- **Date:** 2026-07-12
- **Deciders:** Felipe Carvajal Brown

## Context

"Visual programming for all of Rust" is too broad to ship as a solo developer. The original draft PRD's own success-criteria example (web request → parse JSON → print) already implied a narrower domain than the rest of the draft suggested.

## Decision

MVP targets simple command-line utilities only: read input, transform data, print/write output.

**In scope (MVP):**
- Structs, enums, functions, pattern matching
- `Result`/`Option`, basic error handling
- File I/O, string/collection processing
- Simple synchronous HTTP calls (e.g. one GET request)

**Out of scope (MVP, deferred to post-MVP/1.0):**
- async/await, multithreading
- GUI/windowed output
- Networking servers, complex crate integrations
- Custom trait/generic authoring

## Consequences

- Phase 4 (Standard Library Nodes) only needs to cover this narrower domain to hit MVP, not the full language surface.
- GUI output and async are deferred to 1.0, not dropped — see [ADR-0005](0005-target-agnostic-ir-rust-primary.md) and the roadmap's Post-MVP section.

## Alternatives considered

- **Data pipeline / ETL scripts only** — narrower still ("a visual jq/awk for Rust"). Rejected as more niche than necessary; general CLI/scripting covers the ETL case as a subset without the extra restriction.
- **Backend/API logic only** — assumes an existing web framework scaffold. Rejected: pulls in networking-server concerns (routing, request lifecycle) that the CLI/scripting scope explicitly defers.
- **No scope cut at all ("visual programming for all of Rust")** — the original framing. Rejected as unshippable by a solo developer in any reasonable timeframe.
