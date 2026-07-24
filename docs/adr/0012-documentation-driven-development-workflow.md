# 0012 — Development workflow: Documentation-Driven Development

- **Status:** Accepted.
- **Date:** 2026-07-24
- **Deciders:** Felipe Carvajal Brown

## Context

The project's default code-level loop so far has been **test-first (TDD)**: write a
failing test, make it pass, refactor. That default comes from the tooling
workflow, not from any ADR — there is no prior decision record pinning TDD, so
this ADR does not supersede one; it sets the workflow explicitly for the first
time.

Two things make test-first an awkward *primary* driver for this project:

- **The design level is already documentation-first.** Every feature goes through
  brainstorm -> design spec (`docs/superpowers/specs/`) -> `writing-plans`, on top
  of a stable `PRD.md`, a phase-based `ROADMAP.md`, and immutable ADRs. The
  durable artifacts that keep a *solo* project coherent across long gaps between
  sessions are these documents, not the test suite. The code-level loop being
  test-first while everything above it is document-first is an inconsistency.
- **Large parts of the editor cannot be meaningfully unit-tested.** The
  `egui`/`eframe`/`winit` shell is validated by a human running the app and
  looking at the screen ([ADR-0009](0009-editor-host-standalone-egui-app.md)'s
  acknowledged windowing constraint). Writing a failing test *first* for
  rendering/interaction code that only a human can truly validate adds ceremony
  without catching the bugs that actually matter there.

Documentation-Driven Development (DDD) is a known, written methodology for exactly
this reordering. Its core premise: *from a user's perspective, if a feature is not
documented it does not exist, and if it is documented incorrectly it is broken.*
Its canonical formulation (the widely-cited "DDD" gist) is explicit that DDD does
**not** discard tests — the sequence is: document first, get the documentation
reviewed, then implement (with tests), and tests verify that the implementation
matches the documented behavior; when requirements change, the documentation
changes *before* the code. Tom Preston-Werner's **Readme-Driven Development** is a
lighter sibling that scopes the up-front writing to a single README and argues it
occupies healthy middle ground between waterfall over-specification and agile
under-documentation. Both are the same move: write the contract first, in prose,
before committing to an implementation.

## Decision

Adopt **Documentation-Driven Development as the project's primary development
workflow**, replacing test-first (TDD) as the default code-level loop. Concretely:

1. **Document before implementing.** The behavior and contract of a unit — a
   crate's public API, a module's responsibility, a user-facing feature — is
   written down first: in the design spec, in the ADR that governs it, in the
   README for user-facing surface, and in `///` doc comments describing the
   contract on the items themselves. Code is written to satisfy that documented
   contract.
2. **Documentation is reviewed before code is written.** Canonical DDD's "seek
   community review" step is adapted for a solo project: Felipe is the reviewer,
   and the existing brainstorming design-review gate and spec-approval step serve
   this role. No implementation starts against unreviewed documentation.
3. **Tests are retained, re-scoped as conformance checks that follow the docs.**
   Tests no longer come *first*; they verify that the implementation matches the
   already-documented behavior. Where behavior is genuinely unit-testable — the
   pure logic in `vr-ir`, `vr-rustgen`, `vr-graph` (lowering, validation), and the
   editor's pure mapping/codegen modules — conformance tests remain **required**.
   Where behavior is GUI/windowing (the `eframe` shell), documented behavior plus
   human on-screen validation replaces unit tests, as ADR-0009 already allows.
4. **On change, documentation changes first.** Any change to a feature updates its
   documentation before its code, and its conformance tests after. Documentation
   and code stay version-matched so the docs always describe the shipped behavior.

The existing document hierarchy is the documentation surface: `PRD.md` (vision),
`ROADMAP.md` (phase status), `docs/adr/` (decisions), `docs/superpowers/specs/`
(per-feature design), README + `///` doc comments (user- and consumer-facing
contract). `writing-plans` and `executing-plans` still apply; plans now lead with
documentation and interface definition, then implementation, then conformance
tests — rather than leading with a failing test.

## Consequences

- The code-level loop is now consistent with the already-document-first design
  process; there is one workflow, not two.
- Documentation stays current by construction: it is the driver, written and
  reviewed before code and updated before code on every change, rather than an
  afterthought that rots. This is the single biggest win for a solo project with
  long gaps between sessions — resuming means reading current docs, not
  reverse-engineering intent from tests.
- The awkwardness of writing failing tests first for GUI code that only a human
  can validate is removed; that code is driven by documented behavior and checked
  on screen.
- The safety net shifts. With tests no longer first, correctness rests on
  documented contracts + conformance tests + human validation. **Conformance tests
  for pure logic remain mandatory** precisely so this shift does not become "no
  tests"; only their ordering and primacy change, not their existence. Skipping
  them for pure logic is a regression risk and is not permitted.
- There is a real failure mode: if the "docs-first on change" discipline slips,
  documentation drifts from code and the whole method's value inverts (a
  confidently-wrong doc is worse than none). Mitigation: keep specs lean, update
  the doc in the same change as the code, and treat a doc/code mismatch as a bug.
- Up-front writing costs time before the first line of code, and over-documenting
  before the code reveals the real shape is a risk. Mitigation: scope documents to
  what is being built now (Readme-Driven-Development-style focus), and let specs
  stay MADR-lite and short rather than waterfall-exhaustive.
- Prior specs written under the old framing (e.g. the vr-editor walking-skeleton
  spec, which labels its module tests "TDD") are read under this workflow: the
  pure modules still get conformance tests; those tests now *follow* the documented
  spec rather than precede the code. No rewrite of accepted specs is required.

## Alternatives considered

- **Keep test-first (TDD) as the primary loop** — rejected: it is inconsistent
  with the project's document-first design process, and it forces test-first
  ceremony onto GUI code whose real validation is a human looking at the screen.
  Its strength (a regression net) is preserved by keeping conformance tests
  mandatory for pure logic.
- **Readme-Driven Development only (Preston-Werner)** — the lighter variant,
  scoping all up-front writing to a single README. Adopted *in spirit* for the
  user-facing surface, but too narrow as the whole workflow: this is a multi-crate
  system with internal contracts (IR shape, lowering rules, validation semantics)
  that need specs and ADRs, not just a README. DDD is the superset that covers
  both.
- **Abandon automated tests entirely** — rejected: the pure logic (`vr-ir`,
  `vr-rustgen`, `vr-graph`, the editor's pure modules) genuinely benefits from
  conformance tests, and the generated-code compile+run check is a load-bearing
  CI gate. DDD reorders tests; it does not delete them.
