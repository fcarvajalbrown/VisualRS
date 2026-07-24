# 0011 — Upstream improvements to dependencies; do not fork

- **Status:** Accepted.
- **Date:** 2026-07-24
- **Deciders:** Felipe Carvajal Brown

## Context

The editor pulls in third-party crates that are still evolving, most notably the
`egui`/`eframe` stack and the `egui_snarl` node-graph canvas
([ADR-0010](0010-node-graph-crate-egui-snarl.md)). In the course of building
`vr-editor` we will likely hit bugs, missing features, or needed version bumps in
those dependencies. There are two ways to handle a change we make to a
dependency: contribute it back to the upstream project, or carry it privately as a
fork or vendored patch.

Private forks and long-lived vendored patches rot: they drift from upstream, they
must be re-applied across upstream releases, and they silently keep improvements
out of the ecosystem the project itself depends on. For a solo, open-source
passion project, a divergent fork is also a maintenance burden with no owner. The
project benefits more from a healthy upstream than from a private patch.

## Decision

When our work produces a fix or improvement to any third-party dependency,
**contribute it back upstream as a pull request** rather than keeping it as a
private fork or a permanent vendored patch. Upstreaming is the default and
expected path for any dependency change we author.

A **vendored patch or temporary fork is a last resort**, used only when an
upstream change we need has not yet merged or released and the work cannot proceed
without it. Any such patch is treated as temporary: it is documented (what it
changes, the upstream PR/issue it corresponds to) and tracked for removal once
upstream ships the change.

## Consequences

- Changes we make to `egui_snarl`, `egui`/`eframe`, or any other dependency start
  as upstream PRs, keeping our dependency tree on released upstream versions
  rather than a fork.
- We accept the latency of upstream review: a needed change may land behind an
  unmerged PR for a while. The vendored-patch escape hatch covers that interim,
  explicitly as a tracked, temporary measure.
- The project maintains no permanent fork of a dependency as a matter of routine.
  A standing fork would require its own justification (a future ADR), not a
  silent divergence.
- This is a cross-cutting process decision, independent of which specific crates
  are chosen; it outlives the `egui_snarl` pick and applies to every dependency.

## Alternatives considered

- **Carry improvements as a private fork/vendored patch by default** — rejected:
  forks drift from upstream, must be re-applied across releases, and keep fixes
  out of the ecosystem the project relies on, for no maintenance owner.
- **Stay strictly on released upstream and never touch a dependency's code** —
  rejected as too rigid: it would block legitimate fixes the project needs. The
  chosen policy still allows making the change, it just routes it through an
  upstream PR (with a tracked temporary patch to bridge the gap).
