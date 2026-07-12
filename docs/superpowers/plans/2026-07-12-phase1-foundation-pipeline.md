# Phase 1: Foundation & Pipeline — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a pure-Rust library pipeline that turns a hardcoded, target-agnostic Typed IR into valid, readable, idiomatic Rust source, with CI proving the generated code compiles against Rust stable/beta/nightly.

**Architecture:** Three-crate Cargo workspace. `vr-ir` defines the target-agnostic Typed IR (data + validation + hardcoded fixtures) with zero Rust-AST dependencies. `vr-rustgen` is the only backend through 1.0: it lowers IR to a `proc-macro2` `TokenStream` via `quote`, validates it parses as a `syn::File`, and formats it with `prettyplease`. `vr-cli` is a thin binary that runs the fixture through the pipeline and emits `.rs`. The "graph → IR" front-end is deferred to Phase 2 (the editor); Phase 1's front-end input is a hand-authored IR fixture.

**Tech Stack:** Rust (edition 2021 for the tool crates and for generated code), `syn` 2 + `quote` 1 + `proc-macro2` 1 (Rust AST generation), `prettyplease` 0.2 (formatting), GitHub Actions (CI).

## Global Constraints

- **Target-agnostic IR (ADR-0005):** `vr-ir` MUST NOT depend on `syn`/`quote`/`proc-macro2` or reference any Rust-AST type. The IR is graph-agnostic AND target-agnostic; only `vr-rustgen` knows about Rust syntax.
- **MVP subset only (ADR-0002):** structs, enums, functions, pattern matching, `Result`/`Option`, file I/O, string/collection processing, one-shot sync operations. NO async, NO GUI, NO networking servers, NO custom trait/generic authoring. Phase 1 does not even need HTTP — the capstone is std-only.
- **AST via libraries, never hand-rolled (PRD "Prior Art"):** all Rust code emission goes through `syn`/`quote`/`proc-macro2`. Do not build a bespoke string-concatenation AST layer.
- **Rust version policy (ADR-0007):** CI validates generated code on stable (hard gate) and smoke-tests beta + nightly (non-blocking). MSRV is pinned in the workspace `Cargo.toml` and documented in the README. Generated code targets edition 2021; adopting a new edition requires a dedicated ADR first.
- **Repo conventions (project `CLAUDE.md`):** no emojis anywhere (code, docs, commits); no AI attribution in commits/PRs/code; Conventional Commits; never open a PR unless explicitly asked; keep the PRD/ROADMAP/ADR structure.
- **Transparent generation (PRD):** generated Rust must be readable and well-structured — this is why `prettyplease` formats the output rather than emitting raw token soup.

---

## File Structure

```
CLAUDE.md                         # project conventions (written BEFORE git init)
.gitignore                        # /target, etc.
rust-toolchain.toml               # pins the dev toolchain channel (stable)
Cargo.toml                        # workspace root; MSRV via workspace.package.rust-version
README.md                         # (exists) + MSRV & Rust-channel policy section
ROADMAP.md                        # (exists) + tutorial-phase stub; Phase 1 flipped to Done at the end
crates/
  ir/                             # vr-ir: the Typed IR
    Cargo.toml
    src/
      lib.rs                      # re-exports + crate docs
      lit.rs                      # Literal
      ops.rs                      # BinaryOp, AssignOp
      ty.rs                       # Type
      item.rs                     # Program, Item, StructDef, EnumDef, Variant, VariantPayload, Field, FunctionDef, Param
      stmt.rs                     # Block, Stmt
      expr.rs                     # Expr, MatchArm, BuiltinOp
      pat.rs                      # Pattern
      validate.rs                 # Program::validate
      fixtures.rs                 # line_report(): the hardcoded capstone program
  rustgen/                        # vr-rustgen: IR -> Rust source (the only backend)
    Cargo.toml
    src/
      lib.rs                      # generate(&Program) -> Result<String, GenError>; GenError
      ty.rs                       # gen_type
      lit.rs                      # gen_literal
      pat.rs                      # gen_pattern
      expr.rs                     # gen_expr, gen_builtin
      stmt.rs                     # gen_stmt, gen_block
      item.rs                     # gen_item, gen_struct, gen_enum, gen_fn
    tests/
      compile.rs                  # generated-code validity: rustc-compile + run the capstone (ADR-0007)
  cli/                            # vr-cli: pipeline binary `vrc`
    Cargo.toml
    src/
      main.rs
    tests/
      cli.rs
.github/
  workflows/
    ci.yml                        # fmt, clippy, build, test across stable/beta/nightly
```

---

### Task 1: Repo conventions and git init

**Files:**
- Create: `CLAUDE.md`
- Create: `.gitignore`
- Modify: `ROADMAP.md` (add tutorial-phase stub)

**Interfaces:**
- Consumes: nothing.
- Produces: a git repository with an initial commit; project conventions on disk for every later task.

- [ ] **Step 1: Write the project `CLAUDE.md`** (must exist before `git init`, per Felipe)

Create `CLAUDE.md`:

```markdown
# Visual Rust — project conventions

These are the repo-local conventions distilled from global preferences. They
apply to everyone (human or agent) working in this repository.

## Commits & PRs
- Use **Conventional Commits** (`feat:`, `fix:`, `docs:`, `test:`, `chore:`,
  `refactor:`, `ci:`), scoped where useful (`feat(ir): ...`).
- Commit per completed task; push as work progresses once a remote is configured.
- **Never open a pull request unless explicitly asked** in that same request.
- **No AI attribution** anywhere: no `Co-Authored-By` trailers, no "Generated
  with ..." lines in commits, PRs, code, or docs.

## Writing
- **No emojis** anywhere: code, comments, docs, commit messages, chat.
- **Outward-facing non-technical prose** (README, announcements, marketing copy)
  must go through the humanizer pass before publishing, to strip AI-writing tells
  (em-dash-as-aside, "not just X, but Y" parallelism, uniform sentence rhythm,
  repeated stock adjectives). Technical docs (ADRs, PRD, code comments, this file)
  are exempt.
- Keep the doc structure: `PRD.md` (stable vision), `ROADMAP.md` (phase status),
  `docs/adr/` (one MADR-lite file per decision, immutable once Accepted; supersede
  via a new ADR). Decisions are settled in the ADRs — do not silently diverge; if
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
- Emit Rust through `syn`/`quote`/`proc-macro2` — never hand-roll an AST/string layer.
```

- [ ] **Step 2: Write `.gitignore`**

Create `.gitignore`:

```gitignore
/target
**/*.rs.bk
*.pdb

# Editor/OS noise
.DS_Store
Thumbs.db
.idea/
.vscode/

# Keep Cargo.lock committed: this workspace ships a binary (vr-cli).
```

- [ ] **Step 3: Add the tutorial-phase stub to `ROADMAP.md`**

Insert this block immediately before the `## Post-MVP -> 1.0` heading in `ROADMAP.md`:

```markdown
### Phase 5b: Guided Tutorials & Learning Track
**Status:** Not Started (scope TBD — pending a dedicated brainstorm with Felipe)
- [ ] Scope this out in depth (interactive, in-editor learning path vs. written
      docs vs. sample-driven walkthroughs — decide via brainstorming skill)
- Placeholder: PRD names students/educators as core users; this phase is where the
  in-depth teaching material lives, beyond Phase 5's "beginner tutorials" bullet.

```

- [ ] **Step 4: Initialize git**

Run:

```bash
git init
git add CLAUDE.md .gitignore ROADMAP.md PRD.md README.md docs/
git status
```

Expected: a clean list showing the untracked docs now staged; `CLAUDE.md` present.

- [ ] **Step 5: Commit**

```bash
git commit -m "chore: bootstrap repo conventions and git

- add project CLAUDE.md (conventions distilled from global prefs)
- add .gitignore (Rust workspace, Cargo.lock kept)
- add Phase 5b tutorial stub to ROADMAP"
```

> Note (execution-time gate): pushing requires a remote, which does not exist
> yet. Before the first `git push`, confirm with Felipe whether to create a
> GitHub remote (and public vs. private). Do not create an outward-facing repo
> unprompted.

---

### Task 2: Cargo workspace and three empty crates

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `rust-toolchain.toml`
- Create: `crates/ir/Cargo.toml`, `crates/ir/src/lib.rs`
- Create: `crates/rustgen/Cargo.toml`, `crates/rustgen/src/lib.rs`
- Create: `crates/cli/Cargo.toml`, `crates/cli/src/main.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: a compiling workspace. Crate names `vr-ir` (lib `vr_ir`), `vr-rustgen` (lib `vr_rustgen`), `vr-cli` (bin `vrc`).

- [ ] **Step 1: Determine and pin MSRV from the local toolchain**

Run:

```bash
rustc --version
```

Take the reported `x.y.z` and use `x.y` (that minor) as the MSRV below. Do not guess — read it from this output.

- [ ] **Step 2: Write the workspace root `Cargo.toml`**

Create `Cargo.toml` (replace `1.MINOR` with the minor from Step 1, e.g. `1.82`):

```toml
[workspace]
resolver = "2"
members = ["crates/ir", "crates/rustgen", "crates/cli"]

[workspace.package]
edition = "2021"
rust-version = "1.MINOR"   # MSRV — keep in sync with README
license = "MIT OR Apache-2.0"
repository = ""            # set once a remote exists

[workspace.dependencies]
syn = { version = "2", features = ["full", "parsing", "printing", "extra-traits"] }
quote = "1"
proc-macro2 = "1"
prettyplease = "0.2"
vr-ir = { path = "crates/ir" }
vr-rustgen = { path = "crates/rustgen" }
```

- [ ] **Step 3: Pin the dev toolchain**

Create `rust-toolchain.toml`:

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

- [ ] **Step 4: Write the three crate manifests**

Create `crates/ir/Cargo.toml`:

```toml
[package]
name = "vr-ir"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Visual Rust: target-agnostic Typed IR"

[lib]
name = "vr_ir"

[dependencies]
# Intentionally empty: the IR is target-agnostic (ADR-0005). No syn/quote here, ever.
```

Create `crates/rustgen/Cargo.toml`:

```toml
[package]
name = "vr-rustgen"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Visual Rust: IR -> Rust source backend"

[lib]
name = "vr_rustgen"

[dependencies]
vr-ir.workspace = true
syn.workspace = true
quote.workspace = true
proc-macro2.workspace = true
prettyplease.workspace = true
```

Create `crates/cli/Cargo.toml`:

```toml
[package]
name = "vr-cli"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Visual Rust: pipeline CLI"

[[bin]]
name = "vrc"
path = "src/main.rs"

[dependencies]
vr-ir.workspace = true
vr-rustgen.workspace = true
```

- [ ] **Step 5: Write minimal crate roots**

Create `crates/ir/src/lib.rs`:

```rust
//! Visual Rust Typed IR: a target-agnostic, graph-agnostic representation of a
//! program. No Rust-AST types appear here (ADR-0005).
```

Create `crates/rustgen/src/lib.rs`:

```rust
//! Visual Rust Rust backend: lowers `vr_ir` into Rust source via syn/quote.
```

Create `crates/cli/src/main.rs`:

```rust
fn main() {
    println!("vrc: Visual Rust pipeline (Phase 1)");
}
```

- [ ] **Step 6: Verify the workspace builds**

Run:

```bash
cargo build --workspace
```

Expected: PASS — three crates compile, `vrc` binary builds.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock rust-toolchain.toml crates
git commit -m "chore: scaffold cargo workspace (vr-ir, vr-rustgen, vr-cli)"
```

---

### Task 3: IR — literals, operators, and types

**Files:**
- Create: `crates/ir/src/lit.rs`, `crates/ir/src/ops.rs`, `crates/ir/src/ty.rs`
- Modify: `crates/ir/src/lib.rs`

**Interfaces:**
- Produces:
  - `Literal` enum: `Int(i128)`, `Float(f64)`, `Bool(bool)`, `Char(char)`, `Str(String)`, `Unit`.
  - `BinaryOp` enum: `Add, Sub, Mul, Div, Rem, Eq, Ne, Lt, Le, Gt, Ge, And, Or`.
  - `AssignOp` enum: `Assign, Add, Sub, Mul, Div, Rem`.
  - `Type` enum: `Unit, Bool, Char, I32, I64, Usize, F64, Str, String, Named(String), Ref { mutable: bool, inner: Box<Type> }, Vec(Box<Type>), Option(Box<Type>), Result(Box<Type>, Box<Type>), Tuple(Vec<Type>)`.

- [ ] **Step 1: Write the failing test**

Create `crates/ir/src/ty.rs` with a test at the bottom (types added in Step 3):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_of_string_and_named_is_constructible() {
        let ty = Type::Result(
            Box::new(Type::Unit),
            Box::new(Type::String),
        );
        match ty {
            Type::Result(ok, err) => {
                assert_eq!(*ok, Type::Unit);
                assert_eq!(*err, Type::String);
            }
            _ => panic!("expected Result"),
        }
    }

    #[test]
    fn ref_carries_mutability() {
        let ty = Type::Ref { mutable: true, inner: Box::new(Type::Str) };
        assert_eq!(ty, Type::Ref { mutable: true, inner: Box::new(Type::Str) });
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-ir`
Expected: FAIL — `Type` not found / module not declared.

- [ ] **Step 3: Write the IR value types**

Create `crates/ir/src/lit.rs`:

```rust
/// A literal value. Target-agnostic: no Rust suffixes or types leak in here.
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Int(i128),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(String),
    Unit,
}
```

Create `crates/ir/src/ops.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Rem,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignOp {
    Assign, Add, Sub, Mul, Div, Rem,
}
```

Prepend to `crates/ir/src/ty.rs` (above the `#[cfg(test)]` block):

```rust
/// A target-agnostic type reference. The Rust backend maps these to concrete
/// Rust types; other backends could map them differently (ADR-0005).
#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Unit,
    Bool,
    Char,
    I32,
    I64,
    Usize,
    F64,
    /// Borrowed text (`&str` in the Rust backend).
    Str,
    /// Owned text (`String` in the Rust backend).
    String,
    /// A user-defined struct or enum, by name.
    Named(String),
    Ref { mutable: bool, inner: Box<Type> },
    Vec(Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
}
```

Replace `crates/ir/src/lib.rs` with:

```rust
//! Visual Rust Typed IR: a target-agnostic, graph-agnostic representation of a
//! program. No Rust-AST types appear here (ADR-0005).

pub mod lit;
pub mod ops;
pub mod ty;

pub use lit::Literal;
pub use ops::{AssignOp, BinaryOp};
pub use ty::Type;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vr-ir`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/ir
git commit -m "feat(ir): add literals, operators, and target-agnostic Type"
```

---

### Task 4: IR — items (program, structs, enums, functions)

**Files:**
- Create: `crates/ir/src/item.rs`
- Modify: `crates/ir/src/lib.rs`

**Interfaces:**
- Consumes: `Type` (Task 3), `Block` (defined here as a forward reference resolved in Task 5 — declare the `stmt` module before this compiles).
- Produces:
  - `Program { items: Vec<Item> }`
  - `Item` enum: `Struct(StructDef)`, `Enum(EnumDef)`, `Function(FunctionDef)`
  - `StructDef { name: String, fields: Vec<Field> }`
  - `Field { name: String, ty: Type }`
  - `EnumDef { name: String, variants: Vec<Variant> }`
  - `Variant { name: String, payload: VariantPayload }`
  - `VariantPayload` enum: `Unit`, `Tuple(Vec<Type>)`, `Struct(Vec<Field>)`
  - `FunctionDef { name: String, params: Vec<Param>, ret: Type, body: Block }`
  - `Param { name: String, ty: Type }`

> Because `FunctionDef.body` is a `Block` (from Task 5's `stmt.rs`), do Task 5's
> `stmt.rs`/`expr.rs`/`pat.rs` module files as empty stubs first if executing
> strictly in order, OR execute Task 5 immediately after this task's types are
> written but before running tests. The test below only touches structs/enums/
> params, so it compiles once `Block` exists.

- [ ] **Step 1: Write the failing test**

Add to the bottom of `crates/ir/src/item.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ty::Type;

    #[test]
    fn struct_with_named_fields() {
        let s = StructDef {
            name: "Report".into(),
            fields: vec![
                Field { name: "total_lines".into(), ty: Type::Usize },
                Field { name: "words".into(), ty: Type::Usize },
            ],
        };
        assert_eq!(s.fields.len(), 2);
        assert_eq!(s.fields[0].name, "total_lines");
    }

    #[test]
    fn enum_with_unit_variants() {
        let e = EnumDef {
            name: "LineKind".into(),
            variants: vec![
                Variant { name: "Blank".into(), payload: VariantPayload::Unit },
                Variant { name: "Comment".into(), payload: VariantPayload::Unit },
                Variant { name: "Content".into(), payload: VariantPayload::Unit },
            ],
        };
        assert_eq!(e.variants.len(), 3);
        assert!(matches!(e.variants[0].payload, VariantPayload::Unit));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-ir item`
Expected: FAIL — `StructDef`/`EnumDef` not found.

- [ ] **Step 3: Write the item types**

Prepend to `crates/ir/src/item.rs` (above the test module):

```rust
use crate::stmt::Block;
use crate::ty::Type;

/// A whole program: a flat list of top-level items. A function named `main`
/// (returning `Unit`) is the entry point in the Rust backend.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Struct(StructDef),
    Enum(EnumDef),
    Function(FunctionDef),
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<Variant>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Variant {
    pub name: String,
    pub payload: VariantPayload,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariantPayload {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<Field>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Type,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}
```

Add to `crates/ir/src/lib.rs` (module list + re-exports):

```rust
pub mod item;
```
```rust
pub use item::{
    EnumDef, Field, FunctionDef, Item, Param, Program, StructDef, Variant, VariantPayload,
};
```

- [ ] **Step 4: Run test to verify it passes** (requires Task 5's `stmt.rs` to exist so `Block` resolves)

Run: `cargo test -p vr-ir item`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/ir
git commit -m "feat(ir): add program, struct, enum, and function items"
```

---

### Task 5: IR — statements, expressions, patterns, builtins

**Files:**
- Create: `crates/ir/src/stmt.rs`, `crates/ir/src/expr.rs`, `crates/ir/src/pat.rs`
- Modify: `crates/ir/src/lib.rs`

**Interfaces:**
- Consumes: `Literal`, `BinaryOp`, `AssignOp` (Task 3).
- Produces:
  - `Block { stmts: Vec<Stmt>, tail: Option<Box<Expr>> }`
  - `Stmt` enum: `Let { name: String, mutable: bool, ty: Option<Type>, value: Expr }`, `Assign { target: Expr, op: AssignOp, value: Expr }`, `ForEach { binding: String, iter: Expr, body: Block }`, `Expr(Expr)`, `Return(Option<Expr>)`
  - `Expr` enum: `Lit(Literal)`, `Var(String)`, `Path(Vec<String>)`, `Field { base: Box<Expr>, name: String }`, `Call { func: Box<Expr>, args: Vec<Expr> }`, `MethodCall { receiver: Box<Expr>, method: String, args: Vec<Expr> }`, `Binary { op: BinaryOp, lhs: Box<Expr>, rhs: Box<Expr> }`, `Ref { mutable: bool, expr: Box<Expr> }`, `StructLit { name: String, fields: Vec<(String, Expr)> }`, `Match { scrutinee: Box<Expr>, arms: Vec<MatchArm> }`, `If { cond: Box<Expr>, then: Block, else_: Option<Box<Expr>> }`, `Block(Block)`, `Try(Box<Expr>)`, `Builtin { op: BuiltinOp, args: Vec<Expr> }`
  - `MatchArm { pattern: Pattern, guard: Option<Expr>, body: Expr }`
  - `BuiltinOp` enum: `PrintLine(String)`, `EPrintLine(String)`, `ReadFileToString`, `NthArg(usize)`, `Exit`
  - `Pattern` enum: `Wildcard`, `Binding(String)`, `Path(Vec<String>)`, `TupleStruct { path: Vec<String>, elems: Vec<Pattern> }`, `Tuple(Vec<Pattern>)`, `Literal(Literal)`

- [ ] **Step 1: Write the failing test**

Add to the bottom of `crates/ir/src/expr.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lit::Literal;
    use crate::ops::BinaryOp;

    #[test]
    fn method_call_chains_are_expressions() {
        let e = Expr::MethodCall {
            receiver: Box::new(Expr::Var("line".into())),
            method: "trim".into(),
            args: vec![],
        };
        match e {
            Expr::MethodCall { method, .. } => assert_eq!(method, "trim"),
            _ => panic!("expected method call"),
        }
    }

    #[test]
    fn try_wraps_a_fallible_expr() {
        let e = Expr::Try(Box::new(Expr::Builtin {
            op: BuiltinOp::ReadFileToString,
            args: vec![Expr::Var("path".into())],
        }));
        assert!(matches!(e, Expr::Try(_)));
    }

    #[test]
    fn binary_greater_than() {
        let e = Expr::Binary {
            op: BinaryOp::Gt,
            lhs: Box::new(Expr::Lit(Literal::Int(1))),
            rhs: Box::new(Expr::Lit(Literal::Int(0))),
        };
        assert!(matches!(e, Expr::Binary { op: BinaryOp::Gt, .. }));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-ir expr`
Expected: FAIL — `Expr`/`BuiltinOp` not found.

- [ ] **Step 3: Write the statement, expression, and pattern types**

Create `crates/ir/src/pat.rs`:

```rust
use crate::lit::Literal;

/// A match/binding pattern. Covers the MVP subset: wildcards, bindings,
/// unit-variant paths, data-carrying variants (`Some(x)`, `Ok(v)`), tuples
/// (including `()`), and literals.
#[derive(Clone, Debug, PartialEq)]
pub enum Pattern {
    Wildcard,
    Binding(String),
    /// A path with no payload, e.g. `LineKind::Blank`.
    Path(Vec<String>),
    /// A path with tuple payload, e.g. `Some(x)`, `Ok(v)`, `Err(e)`.
    TupleStruct { path: Vec<String>, elems: Vec<Pattern> },
    Tuple(Vec<Pattern>),
    Literal(Literal),
}
```

Create `crates/ir/src/stmt.rs`:

```rust
use crate::expr::Expr;
use crate::ops::AssignOp;
use crate::ty::Type;

/// A braced block: zero or more statements, then an optional trailing
/// expression that is the block's value.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail: Option<Box<Expr>>,
}

impl Block {
    pub fn new(stmts: Vec<Stmt>, tail: Option<Expr>) -> Self {
        Block { stmts, tail: tail.map(Box::new) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Let { name: String, mutable: bool, ty: Option<Type>, value: Expr },
    Assign { target: Expr, op: AssignOp, value: Expr },
    ForEach { binding: String, iter: Expr, body: Block },
    Expr(Expr),
    Return(Option<Expr>),
}
```

Prepend to `crates/ir/src/expr.rs` (above the test module):

```rust
use crate::lit::Literal;
use crate::ops::BinaryOp;
use crate::pat::Pattern;
use crate::stmt::Block;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Lit(Literal),
    Var(String),
    /// A multi-segment path, e.g. `LineKind::Blank` or `Ok`.
    Path(Vec<String>),
    Field { base: Box<Expr>, name: String },
    Call { func: Box<Expr>, args: Vec<Expr> },
    MethodCall { receiver: Box<Expr>, method: String, args: Vec<Expr> },
    Binary { op: BinaryOp, lhs: Box<Expr>, rhs: Box<Expr> },
    Ref { mutable: bool, expr: Box<Expr> },
    StructLit { name: String, fields: Vec<(String, Expr)> },
    Match { scrutinee: Box<Expr>, arms: Vec<MatchArm> },
    If { cond: Box<Expr>, then: Block, else_: Option<Box<Expr>> },
    Block(Block),
    /// The `?` operator applied to a fallible expression.
    Try(Box<Expr>),
    /// An abstract, well-known operation. The backend chooses concrete code;
    /// the IR stays free of std-library specifics (ADR-0005).
    Builtin { op: BuiltinOp, args: Vec<Expr> },
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

/// Well-known operations the Rust backend maps to concrete code:
/// - `PrintLine(tmpl)`  -> `println!(tmpl, args...)`
/// - `EPrintLine(tmpl)` -> `eprintln!(tmpl, args...)`
/// - `ReadFileToString` -> `std::fs::read_to_string(arg0).map_err(|e| e.to_string())` : Result<String, String>
/// - `NthArg(n)`        -> `std::env::args().nth(n)` : Option<String>
/// - `Exit`             -> `std::process::exit(arg0)`
#[derive(Clone, Debug, PartialEq)]
pub enum BuiltinOp {
    PrintLine(String),
    EPrintLine(String),
    ReadFileToString,
    NthArg(usize),
    Exit,
}
```

Add to `crates/ir/src/lib.rs` (modules + re-exports):

```rust
pub mod expr;
pub mod pat;
pub mod stmt;
```
```rust
pub use expr::{BuiltinOp, Expr, MatchArm};
pub use pat::Pattern;
pub use stmt::{Block, Stmt};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vr-ir`
Expected: PASS (this also unblocks Task 4's tests, since `Block` now exists).

- [ ] **Step 5: Commit**

```bash
git add crates/ir
git commit -m "feat(ir): add statements, expressions, patterns, and builtins"
```

---

### Task 6: IR — validation

**Files:**
- Create: `crates/ir/src/validate.rs`
- Modify: `crates/ir/src/lib.rs`

**Interfaces:**
- Consumes: all IR types.
- Produces: `impl Program { pub fn validate(&self) -> Result<(), Vec<String>> }` — collects human-readable problems. Phase 1 checks: (a) every `Type::Named(n)` refers to a defined struct or enum; (b) there is exactly one function named `main`; (c) no two top-level items share a name.

- [ ] **Step 1: Write the failing test**

Create `crates/ir/src/validate.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::*;

    fn empty_main() -> FunctionDef {
        FunctionDef {
            name: "main".into(),
            params: vec![],
            ret: Type::Unit,
            body: Block::new(vec![], None),
        }
    }

    #[test]
    fn undefined_named_type_is_reported() {
        let prog = Program {
            items: vec![
                Item::Function(FunctionDef {
                    name: "main".into(),
                    params: vec![Param { name: "r".into(), ty: Type::Named("Ghost".into()) }],
                    ret: Type::Unit,
                    body: Block::new(vec![], None),
                }),
            ],
        };
        let errs = prog.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("Ghost")), "got: {errs:?}");
    }

    #[test]
    fn missing_main_is_reported() {
        let prog = Program { items: vec![] };
        let errs = prog.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("main")), "got: {errs:?}");
    }

    #[test]
    fn well_formed_program_validates() {
        let prog = Program { items: vec![Item::Function(empty_main())] };
        assert!(prog.validate().is_ok());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-ir validate`
Expected: FAIL — `validate` not found.

- [ ] **Step 3: Write the validator**

Prepend to `crates/ir/src/validate.rs`:

```rust
use crate::item::{Item, Program, VariantPayload};
use crate::ty::Type;
use std::collections::HashSet;

impl Program {
    /// Basic well-formedness for Phase 1. Returns all problems found, or `Ok`.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut defined_types: HashSet<&str> = HashSet::new();
        let mut item_names: HashSet<String> = HashSet::new();
        let mut main_count = 0usize;

        for item in &self.items {
            let name = match item {
                Item::Struct(s) => {
                    defined_types.insert(s.name.as_str());
                    s.name.clone()
                }
                Item::Enum(e) => {
                    defined_types.insert(e.name.as_str());
                    e.name.clone()
                }
                Item::Function(f) => {
                    if f.name == "main" {
                        main_count += 1;
                    }
                    f.name.clone()
                }
            };
            if !item_names.insert(name.clone()) {
                errors.push(format!("duplicate top-level item name: `{name}`"));
            }
        }

        if main_count == 0 {
            errors.push("no entry point: expected a function named `main`".into());
        } else if main_count > 1 {
            errors.push(format!("expected exactly one `main`, found {main_count}"));
        }

        // Collect every Type mentioned anywhere in an item's signature surface.
        for item in &self.items {
            match item {
                Item::Struct(s) => {
                    for f in &s.fields {
                        check_named(&f.ty, &defined_types, &mut errors);
                    }
                }
                Item::Enum(e) => {
                    for v in &e.variants {
                        match &v.payload {
                            VariantPayload::Unit => {}
                            VariantPayload::Tuple(tys) => {
                                for t in tys {
                                    check_named(t, &defined_types, &mut errors);
                                }
                            }
                            VariantPayload::Struct(fields) => {
                                for f in fields {
                                    check_named(&f.ty, &defined_types, &mut errors);
                                }
                            }
                        }
                    }
                }
                Item::Function(f) => {
                    for p in &f.params {
                        check_named(&p.ty, &defined_types, &mut errors);
                    }
                    check_named(&f.ret, &defined_types, &mut errors);
                }
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// Walk a type and report any `Named` that is not defined in the program.
fn check_named(ty: &Type, defined: &HashSet<&str>, errors: &mut Vec<String>) {
    match ty {
        Type::Named(n) => {
            if !defined.contains(n.as_str()) {
                errors.push(format!("undefined type: `{n}`"));
            }
        }
        Type::Ref { inner, .. }
        | Type::Vec(inner)
        | Type::Option(inner) => check_named(inner, defined, errors),
        Type::Result(a, b) => {
            check_named(a, defined, errors);
            check_named(b, defined, errors);
        }
        Type::Tuple(items) => {
            for t in items {
                check_named(t, defined, errors);
            }
        }
        _ => {}
    }
}
```

Add `pub mod validate;` to `crates/ir/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vr-ir validate`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/ir
git commit -m "feat(ir): add Program::validate (named-type, entry, name-clash checks)"
```

---

### Task 7: IR — the capstone fixture

**Files:**
- Create: `crates/ir/src/fixtures.rs`
- Modify: `crates/ir/src/lib.rs`

**Interfaces:**
- Consumes: all IR types.
- Produces: `pub fn line_report() -> Program` — the hardcoded, std-only "line report" program that exercises the full MVP subset (struct, enum, functions, match, `Result`/`Option`, `?`, file I/O, string/collection processing, for-loop, compound assignment).

The fixture models this Rust program (the Rust backend will produce an equivalent):

```rust
struct Report { total_lines: usize, blank_lines: usize, comment_lines: usize, content_lines: usize, words: usize }
enum LineKind { Blank, Comment, Content }

fn classify(line: &str) -> LineKind {
    let trimmed = line.trim();
    if trimmed.is_empty() { LineKind::Blank }
    else if trimmed.starts_with('#') { LineKind::Comment }
    else { LineKind::Content }
}

fn build_report(text: &str) -> Report {
    let mut report = Report { total_lines: 0, blank_lines: 0, comment_lines: 0, content_lines: 0, words: 0 };
    for line in text.lines() {
        report.total_lines += 1;
        match classify(line) {
            LineKind::Blank => report.blank_lines += 1,
            LineKind::Comment => report.comment_lines += 1,
            LineKind::Content => { report.content_lines += 1; report.words += line.split_whitespace().count(); }
        }
    }
    report
}

fn run() -> Result<(), String> {
    let path = std::env::args().nth(1)?;            // modeled below via match, see note
    let text = std::fs::read_to_string(&path)?;
    let report = build_report(&text);
    println!("lines: {}", report.total_lines);
    println!("blank: {}", report.blank_lines);
    println!("comment: {}", report.comment_lines);
    println!("content: {}", report.content_lines);
    println!("words: {}", report.words);
    Ok(())
}

fn main() {
    match run() { Ok(()) => {}, Err(e) => { eprintln!("error: {}", e); std::process::exit(1); } }
}
```

> Note on `NthArg`: `std::env::args().nth(1)` is `Option<String>`, which cannot
> take `?` inside a `-> Result` fn directly. Model `run`'s arg step as a `match`
> on `NthArg(1)`: `Some(p) => p`, `None => return Err("usage: report <file>")`.
> This keeps the fixture honest and still exercises `Option` + pattern matching.

- [ ] **Step 1: Write the failing test**

Create `crates/ir/src/fixtures.rs` with the test first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_report_is_valid_and_has_expected_items() {
        let prog = line_report();
        prog.validate().expect("fixture must validate");

        let names: Vec<&str> = prog.items.iter().map(|i| match i {
            crate::Item::Struct(s) => s.name.as_str(),
            crate::Item::Enum(e) => e.name.as_str(),
            crate::Item::Function(f) => f.name.as_str(),
        }).collect();

        assert!(names.contains(&"Report"));
        assert!(names.contains(&"LineKind"));
        assert!(names.contains(&"classify"));
        assert!(names.contains(&"build_report"));
        assert!(names.contains(&"run"));
        assert!(names.contains(&"main"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-ir fixtures`
Expected: FAIL — `line_report` not found.

- [ ] **Step 3: Write the fixture**

Prepend to `crates/ir/src/fixtures.rs`. This is verbose but fully concrete — it is the Phase 1 stand-in for editor output. Use these small helpers plus the item builders:

```rust
use crate::*;

// --- tiny expression helpers to keep the fixture readable -------------------
fn var(name: &str) -> Expr { Expr::Var(name.into()) }
fn field(base: Expr, name: &str) -> Expr { Expr::Field { base: Box::new(base), name: name.into() } }
fn path(segs: &[&str]) -> Expr { Expr::Path(segs.iter().map(|s| s.to_string()).collect()) }
fn int(v: i128) -> Expr { Expr::Lit(Literal::Int(v)) }
fn method(recv: Expr, m: &str, args: Vec<Expr>) -> Expr {
    Expr::MethodCall { receiver: Box::new(recv), method: m.into(), args }
}

/// The hardcoded "line report" program: reads a file path from argv, counts
/// blank/comment/content lines and words, prints a report. Std-only.
pub fn line_report() -> Program {
    Program {
        items: vec![
            report_struct(),
            line_kind_enum(),
            classify_fn(),
            build_report_fn(),
            run_fn(),
            main_fn(),
        ],
    }
}

fn report_struct() -> Item {
    let f = |n: &str| Field { name: n.into(), ty: Type::Usize };
    Item::Struct(StructDef {
        name: "Report".into(),
        fields: vec![
            f("total_lines"), f("blank_lines"), f("comment_lines"),
            f("content_lines"), f("words"),
        ],
    })
}

fn line_kind_enum() -> Item {
    let v = |n: &str| Variant { name: n.into(), payload: VariantPayload::Unit };
    Item::Enum(EnumDef {
        name: "LineKind".into(),
        variants: vec![v("Blank"), v("Comment"), v("Content")],
    })
}

fn classify_fn() -> Item {
    // let trimmed = line.trim();
    let let_trimmed = Stmt::Let {
        name: "trimmed".into(), mutable: false, ty: None,
        value: method(var("line"), "trim", vec![]),
    };
    // else { LineKind::Content }
    let else_content = Expr::Block(Block::new(vec![], Some(path(&["LineKind", "Content"]))));
    // else if trimmed.starts_with('#') { Comment } else { Content }
    let inner_if = Expr::If {
        cond: Box::new(method(var("trimmed"), "starts_with",
            vec![Expr::Lit(Literal::Char('#'))])),
        then: Block::new(vec![], Some(path(&["LineKind", "Comment"]))),
        else_: Some(Box::new(else_content)),
    };
    // if trimmed.is_empty() { Blank } else { inner_if }
    let outer_if = Expr::If {
        cond: Box::new(method(var("trimmed"), "is_empty", vec![])),
        then: Block::new(vec![], Some(path(&["LineKind", "Blank"]))),
        else_: Some(Box::new(inner_if)),
    };
    Item::Function(FunctionDef {
        name: "classify".into(),
        params: vec![Param { name: "line".into(), ty: Type::Str }],
        ret: Type::Named("LineKind".into()),
        body: Block::new(vec![let_trimmed], Some(outer_if)),
    })
}

fn build_report_fn() -> Item {
    // let mut report = Report { total_lines: 0, ... };
    let init = Stmt::Let {
        name: "report".into(), mutable: true, ty: None,
        value: Expr::StructLit {
            name: "Report".into(),
            fields: vec![
                ("total_lines".into(), int(0)),
                ("blank_lines".into(), int(0)),
                ("comment_lines".into(), int(0)),
                ("content_lines".into(), int(0)),
                ("words".into(), int(0)),
            ],
        },
    };

    let bump = |f: &str| Stmt::Assign {
        target: field(var("report"), f),
        op: AssignOp::Add,
        value: int(1),
    };

    // Content arm body: { report.content_lines += 1; report.words += line.split_whitespace().count(); }
    let content_body = Expr::Block(Block::new(
        vec![
            bump("content_lines"),
            Stmt::Assign {
                target: field(var("report"), "words"),
                op: AssignOp::Add,
                value: method(
                    method(var("line"), "split_whitespace", vec![]),
                    "count", vec![],
                ),
            },
        ],
        None,
    ));

    let arm = |variant: &str, body: Expr| MatchArm {
        pattern: Pattern::Path(vec!["LineKind".into(), variant.into()]),
        guard: None,
        body,
    };

    let match_stmt = Stmt::Expr(Expr::Match {
        scrutinee: Box::new(Expr::Call {
            func: Box::new(var("classify")),
            args: vec![var("line")],
        }),
        arms: vec![
            arm("Blank", Expr::Assign_placeholder_blank()),   // see note below
        ],
    });
    let _ = match_stmt; // replaced below to avoid the placeholder

    // Build the real match (arms need statement-shaped bodies for Blank/Comment):
    let real_match = Stmt::Expr(Expr::Match {
        scrutinee: Box::new(Expr::Call {
            func: Box::new(var("classify")),
            args: vec![var("line")],
        }),
        arms: vec![
            MatchArm {
                pattern: Pattern::Path(vec!["LineKind".into(), "Blank".into()]),
                guard: None,
                body: Expr::Block(Block::new(vec![bump("blank_lines")], None)),
            },
            MatchArm {
                pattern: Pattern::Path(vec!["LineKind".into(), "Comment".into()]),
                guard: None,
                body: Expr::Block(Block::new(vec![bump("comment_lines")], None)),
            },
            arm("Content", content_body),
        ],
    });

    // for line in text.lines() { report.total_lines += 1; match ... }
    let for_loop = Stmt::ForEach {
        binding: "line".into(),
        iter: method(var("text"), "lines", vec![]),
        body: Block::new(vec![bump("total_lines"), real_match], None),
    };

    Item::Function(FunctionDef {
        name: "build_report".into(),
        params: vec![Param { name: "text".into(), ty: Type::Str }],
        ret: Type::Named("Report".into()),
        body: Block::new(vec![init, for_loop], Some(var("report"))),
    })
}

fn run_fn() -> Item {
    // let path = match std::env::args().nth(1) { Some(p) => p, None => return Err(...) };
    let path_let = Stmt::Let {
        name: "path".into(), mutable: false, ty: None,
        value: Expr::Match {
            scrutinee: Box::new(Expr::Builtin { op: BuiltinOp::NthArg(1), args: vec![] }),
            arms: vec![
                MatchArm {
                    pattern: Pattern::TupleStruct {
                        path: vec!["Some".into()],
                        elems: vec![Pattern::Binding("p".into())],
                    },
                    guard: None,
                    body: var("p"),
                },
                MatchArm {
                    pattern: Pattern::Path(vec!["None".into()]),
                    guard: None,
                    body: Expr::Block(Block::new(
                        vec![Stmt::Return(Some(Expr::Call {
                            func: Box::new(path(&["Err"])),
                            args: vec![method(
                                Expr::Lit(Literal::Str("usage: report <file>".into())),
                                "to_string", vec![],
                            )],
                        }))],
                        None,
                    )),
                },
            ],
        },
    };

    // let text = <read file>?;
    let text_let = Stmt::Let {
        name: "text".into(), mutable: false, ty: None,
        value: Expr::Try(Box::new(Expr::Builtin {
            op: BuiltinOp::ReadFileToString,
            args: vec![Expr::Ref { mutable: false, expr: Box::new(var("path")) }],
        })),
    };

    // let report = build_report(&text);
    let report_let = Stmt::Let {
        name: "report".into(), mutable: false, ty: None,
        value: Expr::Call {
            func: Box::new(var("build_report")),
            args: vec![Expr::Ref { mutable: false, expr: Box::new(var("text")) }],
        },
    };

    let print = |label: &str, f: &str| Stmt::Expr(Expr::Builtin {
        op: BuiltinOp::PrintLine(format!("{label}: {{}}")),
        args: vec![field(var("report"), f)],
    });

    Item::Function(FunctionDef {
        name: "run".into(),
        params: vec![],
        ret: Type::Result(Box::new(Type::Unit), Box::new(Type::String)),
        body: Block::new(
            vec![
                path_let, text_let, report_let,
                print("lines", "total_lines"),
                print("blank", "blank_lines"),
                print("comment", "comment_lines"),
                print("content", "content_lines"),
                print("words", "words"),
            ],
            Some(Expr::Call { func: Box::new(path(&["Ok"])), args: vec![Expr::Lit(Literal::Unit)] }),
        ),
    })
}

fn main_fn() -> Item {
    // match run() { Ok(()) => {}, Err(e) => { eprintln!("error: {}", e); exit(1); } }
    let match_expr = Expr::Match {
        scrutinee: Box::new(Expr::Call { func: Box::new(var("run")), args: vec![] }),
        arms: vec![
            MatchArm {
                pattern: Pattern::TupleStruct { path: vec!["Ok".into()], elems: vec![Pattern::Tuple(vec![])] },
                guard: None,
                body: Expr::Block(Block::new(vec![], None)),
            },
            MatchArm {
                pattern: Pattern::TupleStruct { path: vec!["Err".into()], elems: vec![Pattern::Binding("e".into())] },
                guard: None,
                body: Expr::Block(Block::new(
                    vec![
                        Stmt::Expr(Expr::Builtin {
                            op: BuiltinOp::EPrintLine("error: {}".into()),
                            args: vec![var("e")],
                        }),
                        Stmt::Expr(Expr::Builtin { op: BuiltinOp::Exit, args: vec![int(1)] }),
                    ],
                    None,
                )),
            },
        ],
    };
    Item::Function(FunctionDef {
        name: "main".into(),
        params: vec![],
        ret: Type::Unit,
        body: Block::new(vec![Stmt::Expr(match_expr)], None),
    })
}
```

> Cleanup note for the implementer: the `arm(...)`/`Assign_placeholder_blank()`
> scaffolding line in `build_report_fn` above is illustrative — delete the
> `match_stmt`/`let _ =` placeholder pair and keep only `real_match`. The
> placeholder is called out here so you don't leave dead code; there is no
> `Expr::Assign_placeholder_blank` method to implement.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vr-ir fixtures`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/ir
git commit -m "feat(ir): add hardcoded line-report capstone fixture"
```

---

### Task 8: rustgen — types, literals, paths, and the generate() skeleton

**Files:**
- Create: `crates/rustgen/src/ty.rs`, `crates/rustgen/src/lit.rs`
- Modify: `crates/rustgen/src/lib.rs`

**Interfaces:**
- Consumes: `vr_ir::{Type, Literal}`.
- Produces:
  - `pub fn generate(program: &vr_ir::Program) -> Result<String, GenError>`
  - `pub enum GenError { Validation(Vec<String>), Parse(String) }` (+ `Display`)
  - internal `fn gen_type(&Type) -> proc_macro2::TokenStream`, `fn gen_literal(&Literal) -> proc_macro2::TokenStream`, `fn gen_path(&[String]) -> proc_macro2::TokenStream`.

- [ ] **Step 1: Write the failing test**

Add to `crates/rustgen/src/lib.rs` (test module):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use vr_ir::*;

    #[test]
    fn generates_an_empty_main() {
        let prog = Program {
            items: vec![Item::Function(FunctionDef {
                name: "main".into(), params: vec![], ret: Type::Unit,
                body: Block::new(vec![], None),
            })],
        };
        let src = generate(&prog).expect("should generate");
        assert!(src.contains("fn main()"), "got:\n{src}");
    }

    #[test]
    fn invalid_program_returns_validation_error() {
        let prog = Program { items: vec![] }; // no main
        let err = generate(&prog).unwrap_err();
        assert!(matches!(err, GenError::Validation(_)));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-rustgen`
Expected: FAIL — `generate` not found.

- [ ] **Step 3: Write literals, types, and the skeleton**

Create `crates/rustgen/src/lit.rs`:

```rust
use proc_macro2::TokenStream;
use quote::quote;
use vr_ir::Literal;

pub(crate) fn gen_literal(lit: &Literal) -> TokenStream {
    match lit {
        Literal::Int(v) => {
            let l = proc_macro2::Literal::i128_unsuffixed(*v);
            quote!(#l)
        }
        Literal::Float(v) => {
            let l = proc_macro2::Literal::f64_unsuffixed(*v);
            quote!(#l)
        }
        Literal::Bool(b) => quote!(#b),
        Literal::Char(c) => quote!(#c),
        Literal::Str(s) => quote!(#s),
        Literal::Unit => quote!(()),
    }
}
```

Create `crates/rustgen/src/ty.rs`:

```rust
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use vr_ir::Type;

pub(crate) fn gen_type(ty: &Type) -> TokenStream {
    match ty {
        Type::Unit => quote!(()),
        Type::Bool => quote!(bool),
        Type::Char => quote!(char),
        Type::I32 => quote!(i32),
        Type::I64 => quote!(i64),
        Type::Usize => quote!(usize),
        Type::F64 => quote!(f64),
        Type::Str => quote!(&str),
        Type::String => quote!(String),
        Type::Named(n) => {
            let id = format_ident!("{}", n);
            quote!(#id)
        }
        Type::Ref { mutable, inner } => {
            let inner = gen_type(inner);
            if *mutable { quote!(&mut #inner) } else { quote!(&#inner) }
        }
        Type::Vec(inner) => {
            let inner = gen_type(inner);
            quote!(Vec<#inner>)
        }
        Type::Option(inner) => {
            let inner = gen_type(inner);
            quote!(Option<#inner>)
        }
        Type::Result(ok, err) => {
            let ok = gen_type(ok);
            let err = gen_type(err);
            quote!(Result<#ok, #err>)
        }
        Type::Tuple(items) => {
            let items = items.iter().map(gen_type);
            quote!((#(#items),*))
        }
    }
}

/// A `::`-joined path such as `LineKind::Blank` or `std::process::exit`.
pub(crate) fn gen_path(segs: &[String]) -> TokenStream {
    let idents = segs.iter().map(|s| format_ident!("{}", s));
    quote!(#(#idents)::*)
}
```

Replace `crates/rustgen/src/lib.rs` with (item/expr/stmt/pat modules land in Tasks 9-10; declare them now as empty files or add them as you reach each task):

```rust
//! Visual Rust Rust backend: lowers `vr_ir` into Rust source via syn/quote.

mod lit;
mod ty;
// mod pat;   // Task 10
// mod expr;  // Task 10
// mod stmt;  // Task 10
// mod item;  // Task 9

use std::fmt;

/// Errors from the Rust backend.
#[derive(Debug)]
pub enum GenError {
    /// The IR failed `Program::validate`.
    Validation(Vec<String>),
    /// The emitted tokens did not parse as a `syn::File` (a codegen bug).
    Parse(String),
}

impl fmt::Display for GenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenError::Validation(errs) => {
                writeln!(f, "IR validation failed:")?;
                for e in errs {
                    writeln!(f, "  - {e}")?;
                }
                Ok(())
            }
            GenError::Parse(msg) => write!(f, "generated tokens did not parse: {msg}"),
        }
    }
}

impl std::error::Error for GenError {}

/// Lower a whole program to formatted, readable Rust source.
///
/// Pipeline: validate -> emit `TokenStream` -> parse as `syn::File` (validity
/// gate) -> format with `prettyplease`.
pub fn generate(program: &vr_ir::Program) -> Result<String, GenError> {
    program.validate().map_err(GenError::Validation)?;

    let tokens = emit_program(program);
    let file: syn::File = syn::parse2(tokens)
        .map_err(|e| GenError::Parse(e.to_string()))?;
    Ok(prettyplease::unparse(&file))
}

/// Emit the token stream for the whole file. Fully implemented in Task 9 once
/// item generation exists; the Task 8 skeleton handles only an empty `main`.
fn emit_program(program: &vr_ir::Program) -> proc_macro2::TokenStream {
    use quote::quote;
    // Placeholder until Task 9 wires real item generation:
    let _ = program;
    quote! { fn main() {} }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vr-rustgen`
Expected: PASS — empty-main and validation-error tests green.

- [ ] **Step 5: Commit**

```bash
git add crates/rustgen
git commit -m "feat(rustgen): add generate() skeleton, type and literal emission"
```

---

### Task 9: rustgen — items (structs, enums, function signatures)

**Files:**
- Create: `crates/rustgen/src/item.rs`
- Modify: `crates/rustgen/src/lib.rs`

**Interfaces:**
- Consumes: `gen_type` (Task 8), `gen_block` (Task 10 — see ordering note).
- Produces: `pub(crate) fn gen_item(&Item) -> TokenStream`, and `emit_program` wired to real items.

> Ordering note: `gen_fn` needs `gen_block` from Task 10. Execute Task 10's
> `stmt.rs`/`expr.rs`/`pat.rs` before running this task's full-program tests, or
> stub `gen_block` to `quote!({})` here and replace it in Task 10. The struct/enum
> tests below do not need blocks and pass independently.

- [ ] **Step 1: Write the failing test**

Add to the test module in `crates/rustgen/src/lib.rs`:

```rust
#[test]
fn generates_struct_and_enum() {
    use vr_ir::*;
    let prog = Program {
        items: vec![
            Item::Struct(StructDef {
                name: "Report".into(),
                fields: vec![Field { name: "words".into(), ty: Type::Usize }],
            }),
            Item::Enum(EnumDef {
                name: "LineKind".into(),
                variants: vec![
                    Variant { name: "Blank".into(), payload: VariantPayload::Unit },
                    Variant { name: "Content".into(), payload: VariantPayload::Unit },
                ],
            }),
            Item::Function(FunctionDef {
                name: "main".into(), params: vec![], ret: Type::Unit,
                body: Block::new(vec![], None),
            }),
        ],
    };
    let src = generate(&prog).unwrap();
    assert!(src.contains("struct Report"), "got:\n{src}");
    assert!(src.contains("words: usize"), "got:\n{src}");
    assert!(src.contains("enum LineKind"), "got:\n{src}");
    assert!(src.contains("Blank"), "got:\n{src}");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-rustgen generates_struct_and_enum`
Expected: FAIL — only `fn main() {}` is emitted, no `struct Report`.

- [ ] **Step 3: Write item generation**

Create `crates/rustgen/src/item.rs`:

```rust
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use vr_ir::{EnumDef, Field, FunctionDef, Item, StructDef, VariantPayload};

use crate::stmt::gen_block;
use crate::ty::gen_type;

pub(crate) fn gen_item(item: &Item) -> TokenStream {
    match item {
        Item::Struct(s) => gen_struct(s),
        Item::Enum(e) => gen_enum(e),
        Item::Function(f) => gen_fn(f),
    }
}

fn gen_field(field: &Field) -> TokenStream {
    let name = format_ident!("{}", field.name);
    let ty = gen_type(&field.ty);
    quote!(#name: #ty)
}

fn gen_struct(s: &StructDef) -> TokenStream {
    let name = format_ident!("{}", s.name);
    let fields = s.fields.iter().map(gen_field);
    quote! {
        struct #name {
            #(#fields),*
        }
    }
}

fn gen_enum(e: &EnumDef) -> TokenStream {
    let name = format_ident!("{}", e.name);
    let variants = e.variants.iter().map(|v| {
        let vname = format_ident!("{}", v.name);
        match &v.payload {
            VariantPayload::Unit => quote!(#vname),
            VariantPayload::Tuple(tys) => {
                let tys = tys.iter().map(gen_type);
                quote!(#vname(#(#tys),*))
            }
            VariantPayload::Struct(fields) => {
                let fields = fields.iter().map(gen_field);
                quote!(#vname { #(#fields),* })
            }
        }
    });
    quote! {
        enum #name {
            #(#variants),*
        }
    }
}

fn gen_fn(f: &FunctionDef) -> TokenStream {
    let name = format_ident!("{}", f.name);
    let params = f.params.iter().map(|p| {
        let pname = format_ident!("{}", p.name);
        let ty = gen_type(&p.ty);
        quote!(#pname: #ty)
    });
    let body = gen_block(&f.body);

    // `fn main()` has no return-type arrow when it returns unit; same for any
    // unit-returning fn -> omit `-> ()` for readable output.
    if matches!(f.ret, vr_ir::Type::Unit) {
        quote! {
            fn #name(#(#params),*) #body
        }
    } else {
        let ret = gen_type(&f.ret);
        quote! {
            fn #name(#(#params),*) -> #ret #body
        }
    }
}
```

In `crates/rustgen/src/lib.rs`: uncomment `mod item;` (and the `mod pat/expr/stmt;` lines once Task 10 lands), and replace `emit_program`:

```rust
fn emit_program(program: &vr_ir::Program) -> proc_macro2::TokenStream {
    use quote::quote;
    let items = program.items.iter().map(item::gen_item);
    quote! {
        #(#items)*
    }
}
```

- [ ] **Step 4: Run test to verify it passes** (after Task 10's `gen_block` exists)

Run: `cargo test -p vr-rustgen generates_struct_and_enum`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/rustgen
git commit -m "feat(rustgen): emit structs, enums, and function signatures"
```

---

### Task 10: rustgen — statements, expressions, patterns, builtins

**Files:**
- Create: `crates/rustgen/src/stmt.rs`, `crates/rustgen/src/expr.rs`, `crates/rustgen/src/pat.rs`
- Modify: `crates/rustgen/src/lib.rs` (uncomment `mod pat; mod expr; mod stmt;`)

**Interfaces:**
- Consumes: `gen_type`, `gen_literal`, `gen_path` (Task 8).
- Produces: `pub(crate) fn gen_block(&Block) -> TokenStream`, `gen_stmt`, `gen_expr`, `gen_pattern`, `gen_builtin`.

- [ ] **Step 1: Write the failing test**

Add to the test module in `crates/rustgen/src/lib.rs`:

```rust
#[test]
fn generates_match_try_and_builtins() {
    use vr_ir::*;
    // fn run() -> Result<(), String> {
    //   let text = <read "x">?;
    //   println!("n: {}", 1);
    //   Ok(())
    // }
    let prog = Program {
        items: vec![Item::Function(FunctionDef {
            name: "main".into(),
            params: vec![],
            ret: Type::Unit,
            body: Block::new(
                vec![
                    Stmt::Let {
                        name: "n".into(), mutable: false, ty: None,
                        value: Expr::Binary {
                            op: BinaryOp::Add,
                            lhs: Box::new(Expr::Lit(Literal::Int(1))),
                            rhs: Box::new(Expr::Lit(Literal::Int(2))),
                        },
                    },
                    Stmt::Expr(Expr::Builtin {
                        op: BuiltinOp::PrintLine("n: {}".into()),
                        args: vec![Expr::Var("n".into())],
                    }),
                ],
                None,
            ),
        })],
    };
    let src = generate(&prog).unwrap();
    assert!(src.contains("let n = 1 + 2"), "got:\n{src}");
    assert!(src.contains(r#"println!("n: {}", n)"#), "got:\n{src}");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-rustgen generates_match_try_and_builtins`
Expected: FAIL — `mod stmt` not declared / `gen_block` missing.

- [ ] **Step 3: Write pattern generation**

Create `crates/rustgen/src/pat.rs`:

```rust
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use vr_ir::Pattern;

use crate::lit::gen_literal;
use crate::ty::gen_path;

pub(crate) fn gen_pattern(pat: &Pattern) -> TokenStream {
    match pat {
        Pattern::Wildcard => quote!(_),
        Pattern::Binding(name) => {
            let id = format_ident!("{}", name);
            quote!(#id)
        }
        Pattern::Path(segs) => gen_path(segs),
        Pattern::TupleStruct { path, elems } => {
            let path = gen_path(path);
            let elems = elems.iter().map(gen_pattern);
            quote!(#path(#(#elems),*))
        }
        Pattern::Tuple(elems) => {
            let elems = elems.iter().map(gen_pattern);
            quote!((#(#elems),*))
        }
        Pattern::Literal(lit) => gen_literal(lit),
    }
}
```

- [ ] **Step 4: Write expression and builtin generation**

Create `crates/rustgen/src/expr.rs`:

```rust
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use vr_ir::{BinaryOp, BuiltinOp, Expr, MatchArm};

use crate::lit::gen_literal;
use crate::pat::gen_pattern;
use crate::stmt::gen_block;
use crate::ty::gen_path;

fn gen_binop(op: BinaryOp) -> TokenStream {
    match op {
        BinaryOp::Add => quote!(+), BinaryOp::Sub => quote!(-),
        BinaryOp::Mul => quote!(*), BinaryOp::Div => quote!(/),
        BinaryOp::Rem => quote!(%),
        BinaryOp::Eq => quote!(==), BinaryOp::Ne => quote!(!=),
        BinaryOp::Lt => quote!(<), BinaryOp::Le => quote!(<=),
        BinaryOp::Gt => quote!(>), BinaryOp::Ge => quote!(>=),
        BinaryOp::And => quote!(&&), BinaryOp::Or => quote!(||),
    }
}

pub(crate) fn gen_expr(expr: &Expr) -> TokenStream {
    match expr {
        Expr::Lit(lit) => gen_literal(lit),
        Expr::Var(name) => {
            let id = format_ident!("{}", name);
            quote!(#id)
        }
        Expr::Path(segs) => gen_path(segs),
        Expr::Field { base, name } => {
            let base = gen_expr(base);
            let name = format_ident!("{}", name);
            quote!(#base.#name)
        }
        Expr::Call { func, args } => {
            let func = gen_expr(func);
            let args = args.iter().map(gen_expr);
            quote!(#func(#(#args),*))
        }
        Expr::MethodCall { receiver, method, args } => {
            let receiver = gen_expr(receiver);
            let method = format_ident!("{}", method);
            let args = args.iter().map(gen_expr);
            quote!(#receiver.#method(#(#args),*))
        }
        Expr::Binary { op, lhs, rhs } => {
            let opt = gen_binop(*op);
            let lhs = gen_expr(lhs);
            let rhs = gen_expr(rhs);
            quote!((#lhs #opt #rhs))
        }
        Expr::Ref { mutable, expr } => {
            let inner = gen_expr(expr);
            if *mutable { quote!(&mut #inner) } else { quote!(&#inner) }
        }
        Expr::StructLit { name, fields } => {
            let name = format_ident!("{}", name);
            let fields = fields.iter().map(|(fname, fexpr)| {
                let fname = format_ident!("{}", fname);
                let fexpr = gen_expr(fexpr);
                quote!(#fname: #fexpr)
            });
            quote!(#name { #(#fields),* })
        }
        Expr::Match { scrutinee, arms } => {
            let scrutinee = gen_expr(scrutinee);
            let arms = arms.iter().map(gen_arm);
            quote!(match #scrutinee { #(#arms)* })
        }
        Expr::If { cond, then, else_ } => {
            let cond = gen_expr(cond);
            let then = gen_block(then);
            match else_ {
                Some(e) => {
                    let e = gen_expr(e);
                    quote!(if #cond #then else #e)
                }
                None => quote!(if #cond #then),
            }
        }
        Expr::Block(block) => gen_block(block),
        Expr::Try(inner) => {
            let inner = gen_expr(inner);
            quote!(#inner?)
        }
        Expr::Builtin { op, args } => gen_builtin(op, args),
    }
}

fn gen_arm(arm: &MatchArm) -> TokenStream {
    let pat = gen_pattern(&arm.pattern);
    let body = gen_expr(&arm.body);
    match &arm.guard {
        Some(g) => {
            let g = gen_expr(g);
            quote!(#pat if #g => #body,)
        }
        None => quote!(#pat => #body,),
    }
}

fn gen_builtin(op: &BuiltinOp, args: &[Expr]) -> TokenStream {
    let args_ts: Vec<TokenStream> = args.iter().map(gen_expr).collect();
    match op {
        BuiltinOp::PrintLine(tmpl) => {
            if args_ts.is_empty() { quote!(println!(#tmpl)) }
            else { quote!(println!(#tmpl, #(#args_ts),*)) }
        }
        BuiltinOp::EPrintLine(tmpl) => {
            if args_ts.is_empty() { quote!(eprintln!(#tmpl)) }
            else { quote!(eprintln!(#tmpl, #(#args_ts),*)) }
        }
        BuiltinOp::ReadFileToString => {
            let path = &args_ts[0];
            quote!(std::fs::read_to_string(#path).map_err(|e| e.to_string()))
        }
        BuiltinOp::NthArg(n) => {
            let n = proc_macro2::Literal::usize_unsuffixed(*n);
            quote!(std::env::args().nth(#n))
        }
        BuiltinOp::Exit => {
            let code = &args_ts[0];
            quote!(std::process::exit(#code))
        }
    }
}
```

- [ ] **Step 5: Write statement and block generation**

Create `crates/rustgen/src/stmt.rs`:

```rust
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use vr_ir::{AssignOp, Block, Stmt};

use crate::expr::gen_expr;
use crate::ty::gen_type;

pub(crate) fn gen_block(block: &Block) -> TokenStream {
    let stmts = block.stmts.iter().map(gen_stmt);
    match &block.tail {
        Some(tail) => {
            let tail = gen_expr(tail);
            quote!({ #(#stmts)* #tail })
        }
        None => quote!({ #(#stmts)* }),
    }
}

fn gen_assign_op(op: AssignOp) -> TokenStream {
    match op {
        AssignOp::Assign => quote!(=),
        AssignOp::Add => quote!(+=), AssignOp::Sub => quote!(-=),
        AssignOp::Mul => quote!(*=), AssignOp::Div => quote!(/=),
        AssignOp::Rem => quote!(%=),
    }
}

fn gen_stmt(stmt: &Stmt) -> TokenStream {
    match stmt {
        Stmt::Let { name, mutable, ty, value } => {
            let id = format_ident!("{}", name);
            let value = gen_expr(value);
            let mut_kw = if *mutable { quote!(mut) } else { quote!() };
            match ty {
                Some(t) => {
                    let t = gen_type(t);
                    quote!(let #mut_kw #id: #t = #value;)
                }
                None => quote!(let #mut_kw #id = #value;),
            }
        }
        Stmt::Assign { target, op, value } => {
            let target = gen_expr(target);
            let opt = gen_assign_op(*op);
            let value = gen_expr(value);
            quote!(#target #opt #value;)
        }
        Stmt::ForEach { binding, iter, body } => {
            let binding = format_ident!("{}", binding);
            let iter = gen_expr(iter);
            let body = gen_block(body);
            quote!(for #binding in #iter #body)
        }
        Stmt::Expr(e) => {
            let e = gen_expr(e);
            quote!(#e;)
        }
        Stmt::Return(opt) => match opt {
            Some(e) => { let e = gen_expr(e); quote!(return #e;) }
            None => quote!(return;),
        },
    }
}
```

In `crates/rustgen/src/lib.rs`, ensure the module declarations are active:

```rust
mod lit;
mod ty;
mod pat;
mod expr;
mod stmt;
mod item;
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test -p vr-rustgen`
Expected: PASS — all rustgen unit tests green (including Task 9's struct/enum test now that `gen_block` exists).

- [ ] **Step 7: Commit**

```bash
git add crates/rustgen
git commit -m "feat(rustgen): emit statements, expressions, patterns, and builtins"
```

---

### Task 11: rustgen — full capstone round-trip

**Files:**
- Modify: `crates/rustgen/src/lib.rs` (test only)

**Interfaces:**
- Consumes: `vr_ir::fixtures::line_report`, `generate`.
- Produces: a test asserting the capstone generates, parses as `syn::File`, and contains the expected surface. This is the "IR expresses the MVP subset end-to-end" proof at the source level (compile proof is Task 12).

- [ ] **Step 1: Write the failing test**

Add to the test module in `crates/rustgen/src/lib.rs`:

```rust
#[test]
fn capstone_generates_readable_rust() {
    let prog = vr_ir::fixtures::line_report();
    let src = generate(&prog).expect("capstone must generate");

    // Structural spot-checks across the whole MVP subset:
    assert!(src.contains("struct Report"), "got:\n{src}");
    assert!(src.contains("enum LineKind"), "got:\n{src}");
    assert!(src.contains("fn classify(line: &str) -> LineKind"), "got:\n{src}");
    assert!(src.contains("fn build_report(text: &str) -> Report"), "got:\n{src}");
    assert!(src.contains("fn run() -> Result<(), String>"), "got:\n{src}");
    assert!(src.contains("for line in text.lines()"), "got:\n{src}");
    assert!(src.contains("match"), "got:\n{src}");
    assert!(src.contains("std::fs::read_to_string"), "got:\n{src}");
    assert!(src.contains("std::process::exit(1)"), "got:\n{src}");
    assert!(src.contains("fn main()"), "got:\n{src}");
}
```

- [ ] **Step 2: Run test to verify it fails, then fix codegen until it passes**

Run: `cargo test -p vr-rustgen capstone_generates_readable_rust`
Expected initially: FAIL if any node type in the fixture is not yet handled (a `syn::parse2` error surfaces as `GenError::Parse`, printed by the assert). Fix the offending `gen_*` arm until it passes. Do not weaken the assertions to pass — fix the generator.

- [ ] **Step 3: Commit**

```bash
git add crates/rustgen
git commit -m "test(rustgen): capstone round-trips IR -> readable Rust"
```

---

### Task 12: Generated-code validity — compile and run under the active toolchain

**Files:**
- Create: `crates/rustgen/tests/compile.rs`

**Interfaces:**
- Consumes: `vr_rustgen::generate`, `vr_ir::fixtures::line_report`.
- Produces: an integration test that writes the generated source to a temp dir, compiles it with `rustc` (validity gate), and runs the binary against a sample input (correctness). Because CI runs the test suite under stable/beta/nightly, this single test satisfies ADR-0007's three-channel generated-code validity requirement.

- [ ] **Step 1: Write the failing test**

Create `crates/rustgen/tests/compile.rs`:

```rust
//! Generated-code validity: the capstone must compile with the active `rustc`
//! and run correctly. Under CI's stable/beta/nightly matrix, this is the
//! three-channel validity check required by ADR-0007. Std-only, no network.

use std::process::Command;

/// A unique-enough temp subdir without external crates. Uses PID + a counter
/// baked into the path so parallel test binaries do not collide.
fn temp_dir(tag: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("vr_rustgen_{}_{}", std::process::id(), tag));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into())
}

#[test]
fn capstone_compiles_with_active_rustc() {
    let src = vr_rustgen::generate(&vr_ir::fixtures::line_report()).unwrap();
    let dir = temp_dir("compile");
    let src_path = dir.join("main.rs");
    std::fs::write(&src_path, &src).unwrap();

    // Type/borrow-check without producing a binary: fast validity gate.
    let out = Command::new(rustc())
        .args(["--edition", "2021", "--crate-type", "bin", "--emit=metadata"])
        .arg(&src_path)
        .arg("--out-dir")
        .arg(&dir)
        .output()
        .expect("failed to invoke rustc");

    assert!(
        out.status.success(),
        "generated code failed to compile:\n--- source ---\n{src}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn capstone_runs_and_reports_counts() {
    let src = vr_rustgen::generate(&vr_ir::fixtures::line_report()).unwrap();
    let dir = temp_dir("run");
    let src_path = dir.join("main.rs");
    std::fs::write(&src_path, &src).unwrap();

    let exe = dir.join(if cfg!(windows) { "report.exe" } else { "report" });
    let build = Command::new(rustc())
        .args(["--edition", "2021", "-O"])
        .arg(&src_path)
        .arg("-o")
        .arg(&exe)
        .output()
        .expect("failed to invoke rustc");
    assert!(
        build.status.success(),
        "build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    // Sample input: 1 blank, 1 comment, 2 content lines, 5 content words.
    let input = dir.join("sample.txt");
    std::fs::write(&input, "\n# a comment\nhello world\nthree more words\n").unwrap();

    let run = Command::new(&exe).arg(&input).output().expect("run failed");
    assert!(run.status.success(), "program exited with error: {run:?}");
    let stdout = String::from_utf8_lossy(&run.stdout);

    assert!(stdout.contains("lines: 4"), "got:\n{stdout}");
    assert!(stdout.contains("blank: 1"), "got:\n{stdout}");
    assert!(stdout.contains("comment: 1"), "got:\n{stdout}");
    assert!(stdout.contains("content: 2"), "got:\n{stdout}");
    assert!(stdout.contains("words: 5"), "got:\n{stdout}");
}
```

- [ ] **Step 2: Run tests to verify behavior**

Run: `cargo test -p vr-rustgen --test compile`
Expected: PASS. If the compile test fails, the failure message prints the full generated source and `rustc` diagnostics — fix the generator (or the fixture) per the diagnostic, not the assertion. If counts are off, correct the fixture's logic, not the expected numbers.

- [ ] **Step 3: Commit**

```bash
git add crates/rustgen/tests
git commit -m "test(rustgen): compile and run generated capstone under active rustc"
```

---

### Task 13: vr-cli — the pipeline binary

**Files:**
- Modify: `crates/cli/src/main.rs`
- Create: `crates/cli/tests/cli.rs`

**Interfaces:**
- Consumes: `vr_ir::fixtures::line_report`, `vr_rustgen::generate`.
- Produces: `vrc` — prints the generated capstone source to stdout, or writes it to a file with `--out <path>`. Exits non-zero on generation error.

- [ ] **Step 1: Write the failing test**

Create `crates/cli/tests/cli.rs`:

```rust
use std::process::Command;

fn vrc() -> Command {
    // Cargo sets CARGO_BIN_EXE_<name> for integration tests.
    Command::new(env!("CARGO_BIN_EXE_vrc"))
}

#[test]
fn prints_generated_source_to_stdout() {
    let out = vrc().output().expect("run vrc");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("fn main()"), "got:\n{stdout}");
    assert!(stdout.contains("struct Report"), "got:\n{stdout}");
}

#[test]
fn writes_to_out_path() {
    let dir = std::env::temp_dir().join(format!("vrc_out_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let out_path = dir.join("generated.rs");

    let status = vrc()
        .arg("--out")
        .arg(&out_path)
        .status()
        .expect("run vrc --out");
    assert!(status.success());

    let contents = std::fs::read_to_string(&out_path).unwrap();
    assert!(contents.contains("fn build_report"), "got:\n{contents}");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vr-cli`
Expected: FAIL — stub `main` prints a banner, not the generated source.

- [ ] **Step 3: Implement the binary**

Replace `crates/cli/src/main.rs`:

```rust
//! `vrc`: run the hardcoded Phase 1 capstone through the pipeline
//! (IR -> Rust source) and emit it. `--out <path>` writes to a file;
//! otherwise the source goes to stdout.

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut out_path: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => {
                match args.next() {
                    Some(p) => out_path = Some(p),
                    None => {
                        eprintln!("error: --out requires a path");
                        return ExitCode::from(2);
                    }
                }
            }
            "-h" | "--help" => {
                println!("usage: vrc [--out <path>]");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("error: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    let program = vr_ir::fixtures::line_report();
    let source = match vr_rustgen::generate(&program) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("codegen error: {e}");
            return ExitCode::FAILURE;
        }
    };

    match out_path {
        Some(path) => {
            if let Err(e) = std::fs::write(&path, source) {
                eprintln!("error writing {path}: {e}");
                return ExitCode::FAILURE;
            }
        }
        None => print!("{source}"),
    }
    ExitCode::SUCCESS
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vr-cli`
Expected: PASS.

- [ ] **Step 5: Manually run the pipeline end-to-end**

Run:

```bash
cargo run -p vr-cli -- --out target/generated.rs
rustc --edition 2021 -O target/generated.rs -o target/report
./target/report ./README.md
```

Expected: the `report` binary prints line/word counts for `README.md`.

- [ ] **Step 6: Commit**

```bash
git add crates/cli
git commit -m "feat(cli): vrc emits the generated capstone to stdout or --out"
```

---

### Task 14: CI — GitHub Actions across stable, beta, nightly

**Files:**
- Create: `.github/workflows/ci.yml`
- Modify: `README.md` (add MSRV + Rust-channel policy section)

**Interfaces:**
- Consumes: the whole workspace and its test suite (Task 12's compile/run test is what makes CI channel-sensitive).
- Produces: a CI workflow that gates on stable and smoke-tests beta/nightly per ADR-0007.

- [ ] **Step 1: Write the workflow**

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: ["**"]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  # Hard gate: formatting, lints, build, and full test suite on stable.
  # The generated-code compile+run test (crates/rustgen/tests/compile.rs)
  # runs here under stable rustc, satisfying the stable validity requirement.
  stable:
    name: stable (gate)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install stable toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: Formatting
        run: cargo fmt --all --check
      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: Build
        run: cargo build --workspace --all-targets
      - name: Test
        run: cargo test --workspace

  # Smoke test: catch upcoming breakage early. Non-blocking (ADR-0007): a red
  # beta/nightly is a signal to investigate, not a merge blocker. Per project
  # policy, when a release lands and a channel goes red, stay on it until green.
  smoke:
    name: smoke (${{ matrix.channel }})
    runs-on: ubuntu-latest
    continue-on-error: true
    strategy:
      fail-fast: false
      matrix:
        channel: [beta, nightly]
    steps:
      - uses: actions/checkout@v4
      - name: Install ${{ matrix.channel }} toolchain
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.channel }}
      - uses: Swatinem/rust-cache@v2
      - name: Test (generated-code validity on ${{ matrix.channel }})
        run: cargo test --workspace
```

- [ ] **Step 2: Add the MSRV + policy section to the README**

Append to `README.md`:

```markdown
## Rust version policy

- **MSRV:** pinned in the workspace `Cargo.toml` (`workspace.package.rust-version`).
  Keep this line and the manifest in sync when the floor moves.
- **Generated code** targets **edition 2021**. Adopting a new edition requires a
  dedicated ADR first (see [ADR-0007](docs/adr/0007-rust-version-compatibility-policy.md)).
- **CI:** stable is a hard gate (fmt, clippy, build, test, and the generated-code
  compile+run check). Beta and nightly are non-blocking smoke tests that run the
  same suite to catch breakage before it reaches stable. When a new release turns
  a channel red, that is active work until all three channels are green again.
```

- [ ] **Step 3: Verify the workflow is well-formed and the suite is green locally**

Run:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all PASS. (The workflow itself runs on push once a remote exists; there is no local GitHub Actions execution step here.)

- [ ] **Step 4: Commit**

```bash
git add .github README.md
git commit -m "ci: add stable-gate + beta/nightly smoke workflow; document Rust policy"
```

---

### Task 15: Flip ROADMAP Phase 1 to Done

**Files:**
- Modify: `ROADMAP.md`

**Interfaces:**
- Consumes: completed Tasks 1-14.
- Produces: an accurate roadmap.

- [ ] **Step 1: Update the Phase 1 block**

In `ROADMAP.md`, change the Phase 1 section to:

```markdown
### Phase 1: Foundation & Pipeline
**Status:** Done (completed 2026-07-12)
- [x] Define the Typed IR specification (target-agnostic per [ADR-0005](docs/adr/0005-target-agnostic-ir-rust-primary.md))
- [x] Build the core translation engine: hardcoded graph data -> IR -> Rust AST
- [x] Establish CI: test generated-code validity against stable Rust; smoke-test against beta/nightly per [ADR-0007](docs/adr/0007-rust-version-compatibility-policy.md)
```

> Note: "hardcoded graph data" is realized in Phase 1 as a hand-authored IR
> fixture (`vr_ir::fixtures::line_report`); the real graph->IR lowering begins in
> Phase 2 with the editor, as agreed during planning.

- [ ] **Step 2: Verify the whole workspace one final time**

Run:

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected: all PASS.

- [ ] **Step 3: Commit**

```bash
git add ROADMAP.md
git commit -m "docs: mark ROADMAP Phase 1 (Foundation & Pipeline) done"
```

---

## Self-Review

**1. Spec coverage (Phase 1 checklist + ADRs):**
- "Define the Typed IR specification, target-agnostic" -> Tasks 3-7; `vr-ir` has zero syn/quote deps (Task 2 manifest; Global Constraints). Covered.
- "Core translation engine: hardcoded graph data -> IR -> Rust AST" -> hardcoded input is the IR fixture (Task 7); IR -> Rust AST via syn/quote is Tasks 8-11; end-to-end binary is Task 13. Covered (graph layer deferred to Phase 2 by explicit decision).
- "CI: validity vs stable; smoke beta/nightly" -> Task 14 workflow + Task 12 compile/run test (the channel-sensitive check). Covered.
- ADR-0005 (target-agnostic IR) -> enforced by `vr-ir` having no Rust-AST deps and by builtins abstracting std ops. Covered.
- ADR-0002 (MVP subset) -> the IR expresses exactly the subset; capstone exercises struct/enum/fn/match/Result/Option/file I/O/string+collection processing. Covered.
- ADR-0007 (version policy) -> Task 14 workflow, README policy section, MSRV pin (Task 2). Covered.
- PRD "AST via libraries" -> syn/quote/proc-macro2 + prettyplease; no hand-rolled AST. Covered.
- Out-of-scope (Godot/editor, borrow-viz UI, node library, async/GUI/plugin) -> not touched. Covered.

**2. Placeholder scan:** The only intentional "placeholder" is the `emit_program` stub in Task 8, explicitly replaced in Task 9, and the illustrative dead-code line in Task 7's fixture, explicitly flagged for deletion. No `TBD`/"add error handling"/"write tests for the above" left. The Task 8 `mod` comments are replaced in Tasks 9-10. Acceptable.

**3. Type/name consistency:** `generate(&Program) -> Result<String, GenError>` used identically in Tasks 8/11/12/13. `gen_block` produced in Task 10, consumed in Tasks 9/10. `line_report()` produced in Task 7, consumed in Tasks 11/12/13. Crate/lib names (`vr-ir`/`vr_ir`, `vr-rustgen`/`vr_rustgen`, `vr-cli`/`vrc`) consistent across Tasks 2-14. IR variant names (`BuiltinOp::PrintLine/EPrintLine/ReadFileToString/NthArg/Exit`, `Pattern::TupleStruct`, `Stmt::ForEach/Assign`) match between `vr-ir` definitions (Task 5) and `vr-rustgen` matches (Task 10). Consistent.
```