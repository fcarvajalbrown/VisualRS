# Architecture Decision Records

One file per significant decision. Format is MADR-lite: Status, Date,
Deciders, Context, Decision, Consequences, Alternatives considered.

Once an ADR is **Accepted**, treat it as immutable. To change course, write
a new ADR whose `Status` declares "Supersedes 00XX", and edit the older
ADR's `Status` to "Superseded by 00YY".

## Index

| # | Title | Status |
|---|-------|--------|
| [0001](0001-borrow-violation-visualization.md) | Borrow-checker violation visualization | Accepted |
| [0002](0002-mvp-scope-cli-scripting-only.md) | MVP scope: CLI/scripting only | Accepted |
| [0003](0003-editor-host-platform-godot-gdext.md) | Editor host platform: Godot + gdext | Accepted |
| [0004](0004-flagship-domain-godot-gdext-gamedev.md) | Flagship post-1.0 domain pack: Godot GDExtension | Accepted |
| [0005](0005-target-agnostic-ir-rust-primary.md) | Target-agnostic IR; Rust primary through 1.0 | Accepted |
| [0006](0006-output-gui-toolkit-egui-swappable.md) | Output GUI toolkit: egui, swappable | Accepted |
| [0007](0007-rust-version-compatibility-policy.md) | Rust version/edition compatibility policy | Accepted |
