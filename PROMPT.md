 You are continuing Phase 1 of the Visual Rust project (a visual-programming-to-Rust
  codegen tool). Working directory: C:\Projects\VisualRS (Windows, PowerShell primary,
  Bash tool available; rustc/cargo 1.94.0 stable installed, plus nightly toolchains).

  READ FIRST, in this order, before doing anything else:  1. C:\Projects\VisualRS\docs\superpowers\plans\2026-07-12-phase1-foundation-pipeline.md
     — the full 15-task TDD implementation plan. This is your script. Follow it task by task.  2. CLAUDE.md (repo root) — project conventions. Obey them exactly.
  3. PRD.md, ROADMAP.md, and docs/adr/*.md — the settled product spec and architecture
     decisions. Do NOT re-derive or second-guess them. If something seems wrong or you need
     to deviate from a decision, STOP and ask the user rather than silently diverging.

  WHAT IS ALREADY DONE (Tasks 1-7 of the plan, all committed + pushed):
  - Git repo is initialized; remote origin = https://github.com/fcarvajalbrown/VisualRS
    (public). You are on branch `phase-1-foundation` (tracks origin). `main` is untouched.
  - Cargo workspace with 3 crates. MSRV pinned to 1.94 in root Cargo.toml. Generated code
    targets edition 2021.
  - crates/ir (vr-ir): the target-agnostic Typed IR, COMPLETE. Modules: lit, ops, ty, pat,
    stmt, expr, item, validate, fixtures. `Program::validate()` works. `fixtures::line_report()`
    is the hardcoded capstone program. 11 unit tests pass; fmt + clippy -D warnings clean.
    vr-ir has ZERO syn/quote/proc-macro2 dependency and MUST stay that way (ADR-0005).
  - crates/rustgen (vr-rustgen) and crates/cli (vr-cli) currently contain only stub lib.rs/
    main.rs from the workspace scaffold. This is where your work starts.

  YOUR JOB: execute Tasks 8 through 15 of the plan, in order.
  - Task 8: vr-rustgen types/literals/paths + generate() skeleton (syn parse2 validity +
    prettyplease formatting).
  - Task 9: vr-rustgen items (structs, enums, fn signatures).
  - Task 10: vr-rustgen statements, expressions, patterns, builtins.
  - Task 11: full capstone round-trip test (generate the fixture, assert it parses).
  - Task 12: crates/rustgen/tests/compile.rs — write generated source, compile with rustc,
    and RUN it against a sample input asserting the output counts. This is the ADR-0007
    generated-code validity check.
  - Task 13: vr-cli binary `vrc` — emit the generated capstone to stdout or --out <path>.
  - Task 14: .github/workflows/ci.yml (stable hard gate: fmt+clippy+build+test; beta+nightly
    smoke via continue-on-error) + a Rust-version-policy section appended to README.md.
  - Task 15: flip ROADMAP.md Phase 1 to Status: Done and check its 3 boxes.

  Note: the plan's Task 8/9 have module-ordering caveats (rustgen item gen needs gen_block
  from Task 10). Land Tasks 8-10 so the crate compiles as a unit, exactly as the plan
  describes. The plan contains complete, concrete code for every step — use it; do not
  improvise a different IR or codegen shape.

  HOW TO WORK:
  - Use the superpowers:executing-plans skill (inline execution, batched with checkpoints).
    Invoke test-driven-development discipline per the plan (test, see it fail if practical,
    implement, see it pass, commit).
  - Verify each task before marking it done: `cargo test -p <crate>`, and before committing a
    crate run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` (CI enforces
    these). Never claim a task passes without running it.
  - Commit per task with Conventional Commits messages (feat:/test:/ci:/docs:/chore:). Commit
    AND push to `phase-1-foundation` as you go. Do NOT merge to main and do NOT open a pull
    request unless the user explicitly asks in that turn.
  - After Task 12 and again after Task 15, pause and give the user a concise checkpoint
    summary (what passed, what's next) — they are running this batched-with-checkpoints.
  - When all 15 tasks are green, use the superpowers:finishing-a-development-branch skill to
    present integration options (merge to main / PR / leave as-is) — let the user choose;
    don't merge or PR on your own initiative.

  HARD RULES (from CLAUDE.md — these override defaults):
  - No emojis anywhere (code, docs, commits, chat).
  - No AI attribution anywhere: no "Co-Authored-By", no "Generated with Claude" lines.
  - Never open a PR unless explicitly asked this turn.
  - Outward-facing NON-technical prose (e.g. README marketing copy) must go through the
    humanizer skill before publishing; technical docs (ADRs, PRD, code, the README's new
    Rust-policy section) are exempt. The README intro was already humanized — don't undo it.
  - Present any decision to the user via the interactive option UI (blue selector), one option
    marked "(Recommended)" and first, with the reasoning stated. Ask before decision-affecting
    or hard-to-reverse actions.
  - vr-ir must never gain a syn/quote/proc-macro2 dependency. Only vr-rustgen knows Rust syntax.

  There is a harness task list (#1-#15) already tracking these; tasks #1-7 are completed.
  Update task status as you progress (in_progress when starting, completed when verified).

  Start by reading the plan file, then continue at Task 8.