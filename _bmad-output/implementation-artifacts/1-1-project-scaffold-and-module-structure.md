# Story 1.1: Project Scaffold & Module Structure

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want the project scaffold organized to match the target architecture with all dependencies in place,
so that all subsequent stories have the correct structure and libraries to build upon.

## Acceptance Criteria

1. **Given** the scaffold setup is complete, **When** the project is built, **Then** the build succeeds with no errors or warnings **And** the module structure matches the architecture document.

## Tasks / Subtasks

- [x] Task 1: Add missing Cargo.toml dependencies (AC: 1)
  - [x] Add `toml = "~0.8"` to `[dependencies]`
  - [x] Add `signal-hook = "~0.3"` to `[dependencies]`
  - [x] Verify all 8 required crates are present: ratatui, crossterm, dashu, serde+serde_json, toml, dirs, arboard, signal-hook

- [x] Task 2: Complete the engine module structure (AC: 1)
  - [x] Create `src/engine/error.rs` — minimal `CalcError` enum with at least `StackUnderflow`, `DivisionByZero`, `DomainError(String)`, `InvalidInput(String)`, `NotAnInteger` variants; derive `Debug`; implement `std::fmt::Display`
  - [x] Create `src/engine/registers.rs` — stub file; can be empty except for a `// TODO: Story 1.2` comment to ensure it compiles
  - [x] Update `src/engine/mod.rs` — add `pub mod error;` and `pub mod registers;` declarations alongside existing ones; add `pub use error::CalcError;` re-export

- [x] Task 3: Create the input module structure (AC: 1)
  - [x] Create `src/input/mod.rs` — declare all submodules: `pub mod action; pub mod commands; pub mod handler; pub mod mode; pub mod parser;`
  - [x] Create `src/input/action.rs` — stub `Action` enum with `#[allow(dead_code)]` and a single placeholder variant `Noop`; derive `Debug, PartialEq`
  - [x] Create `src/input/mode.rs` — stub `AppMode` enum with variants `Normal`, `Alpha(String)`, `Chord(ChordCategory)`; stub `ChordCategory` enum with variants `Trig`, `Log`, `Functions`, `Constants`, `AngleMode`, `Base`, `HexStyle`; derive `Debug, Clone, PartialEq` on both
  - [x] Create `src/input/handler.rs` — stub `pub fn handle_key(_mode: &AppMode, _event: crossterm::event::KeyEvent) -> Action` that returns `Action::Noop`; add `use crate::input::{action::Action, mode::AppMode};`
  - [x] Create `src/input/commands.rs` — stub `pub fn parse_command(_input: &str) -> Result<Action, crate::engine::CalcError>` that returns `Err(crate::engine::CalcError::InvalidInput("not implemented".to_string()))`; add appropriate use statements

- [x] Task 4: Create the tui module structure (AC: 1)
  - [x] Create `src/tui/mod.rs` — declare `pub mod app; pub mod layout; pub mod widgets;`
  - [x] Create `src/tui/app.rs` — stub `App` struct with at minimum a `pub should_quit: bool` field and `pub fn new() -> Self` constructor; add `#[allow(dead_code)]` on the struct
  - [x] Create `src/tui/layout.rs` — stub file with a `// TODO: Story 2.1` comment
  - [x] Create `src/tui/widgets/mod.rs` — declare `pub mod error_line; pub mod hints_pane; pub mod input_line; pub mod mode_bar; pub mod stack_pane;`
  - [x] Create `src/tui/widgets/stack_pane.rs` — stub file with a `// TODO: Story 2.2` comment
  - [x] Create `src/tui/widgets/input_line.rs` — stub file with a `// TODO: Story 2.4` comment
  - [x] Create `src/tui/widgets/hints_pane.rs` — stub file with a `// TODO: Story 3.1` comment
  - [x] Create `src/tui/widgets/error_line.rs` — stub file with a `// TODO: Story 2.3` comment
  - [x] Create `src/tui/widgets/mode_bar.rs` — stub file with a `// TODO: Story 2.3` comment

- [x] Task 5: Create the config module structure (AC: 1)
  - [x] Create `src/config/mod.rs` — declare `pub mod config; pub mod session;`
  - [x] Create `src/config/config.rs` — stub `Config` struct with `#[allow(dead_code)]` and a `pub fn load() -> Self` stub returning `Config::default()`; implement `Default` with sensible defaults (DEG, DEC, precision 15, undo depth 1000, persist_session true)
  - [x] Create `src/config/session.rs` — stub file with a `// TODO: Story 4.3` comment; add a `pub fn save() -> Result<(), std::io::Error> { Ok(()) }` stub

- [x] Task 6: Create the tests directory structure (AC: 1)
  - [x] Create `tests/integration/` directory
  - [x] Create `tests/integration/session_roundtrip.rs` — minimal placeholder: `#[test] fn placeholder() {}`
  - [x] Create `tests/integration/keybinding_sequences.rs` — minimal placeholder: `#[test] fn placeholder() {}`

- [x] Task 7: Update main.rs to declare all top-level modules (AC: 1)
  - [x] Replace `fn main()` body content, keeping the function; declare `mod engine; mod input; mod tui; mod config;` at the top of main.rs; main body can remain `println!("Hello, world!");` for now

- [x] Task 8: Fix any clippy warnings in existing files (AC: 1)
  - [x] Run `cargo clippy -- -D warnings` and fix any warnings
  - [x] Run `cargo fmt`
  - [x] **Do NOT change any logic** in existing files — only fix lints (e.g. add `#[allow(...)]` for false positives, fix naming, fix unused import warnings)
  - [x] Existing files that likely need attention: `engine/value.rs` (unwrap() calls), `engine/undo.rs` (snapshots Vec<CalcValue> not full CalcState — acceptable for now, Story 1.2 will replace), `input/parser.rs` (uses String errors not CalcError — acceptable for now, Story 1.5 will replace)

- [x] Task 9: Verify all quality gates pass (AC: 1)
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt --check` exits 0 (or `cargo fmt` applied)
  - [x] `cargo test` exits 0 (existing tests in `input/parser.rs` must still pass)

## Dev Notes

### Architectural Context

This story is a **scaffold-only** story. Its sole purpose is to establish the correct directory and module skeleton so that Stories 1.2–4.4 can add real implementations to the right places. No business logic is implemented here.

**Critical principle:** CalcState is the sole mutable state in the final architecture. The existing `StackEngine` struct in `engine/stack.rs` is a legacy structure that will be replaced in Story 1.2. Do NOT refactor `stack.rs`, `ops.rs`, or `undo.rs` in this story — leave their implementations intact even if they differ from the target architecture. Story 1.2 will define `CalcState` and subsequent stories will update these files.

**Module hierarchy (final target):**
```
src/
  main.rs                  ← declares: mod engine; mod input; mod tui; mod config;
  engine/
    mod.rs                 ← pub mod angle, base, constants, error, ops, registers, stack, undo, value
                             pub use error::CalcError;
    error.rs               ← CalcError enum  ← CREATE in this story
    registers.rs           ← register store/recall  ← CREATE stub in this story
    angle.rs               ← AngleMode  (exists ✓)
    base.rs                ← Base, (HexStyle to be added in Story 1.2)  (exists ✓)
    value.rs               ← CalcValue  (exists ✓, Serialize/Deserialize to be added in Story 1.2)
    stack.rs               ← stack ops  (exists, will be updated in Stories 1.2-1.3)
    ops.rs                 ← operations  (exists, will be updated in Story 1.4)
    undo.rs                ← UndoHistory  (exists, will be updated in Story 1.2)
    constants.rs           ← π, e, φ  (exists ✓)
  input/
    mod.rs                 ← CREATE in this story
    action.rs              ← CREATE stub in this story
    mode.rs                ← CREATE stub in this story
    handler.rs             ← CREATE stub in this story
    commands.rs            ← CREATE stub in this story
    parser.rs              ← parse_value  (exists ✓)
  tui/
    mod.rs                 ← CREATE in this story
    app.rs                 ← CREATE stub in this story
    layout.rs              ← CREATE stub in this story
    widgets/
      mod.rs               ← CREATE in this story
      stack_pane.rs        ← CREATE stub in this story
      input_line.rs        ← CREATE stub in this story
      hints_pane.rs        ← CREATE stub in this story
      error_line.rs        ← CREATE stub in this story
      mode_bar.rs          ← CREATE stub in this story
  config/
    mod.rs                 ← CREATE in this story
    config.rs              ← CREATE stub in this story
    session.rs             ← CREATE stub in this story
tests/
  integration/
    session_roundtrip.rs   ← CREATE placeholder in this story
    keybinding_sequences.rs← CREATE placeholder in this story
```

### Stub Code Patterns

Stubs must compile cleanly. Recommended patterns:

**For enum stubs with no uses yet:**
```rust
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Noop,
}
```

**For function stubs that aren't called yet:**
```rust
#[allow(dead_code)]
pub fn handle_key(_mode: &AppMode, _event: crossterm::event::KeyEvent) -> Action {
    Action::Noop
}
```

**For struct stubs:**
```rust
#[allow(dead_code)]
pub struct App {
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self { should_quit: false }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
```

**For empty module files:** A single comment is sufficient: `// TODO: Story X.Y`

### CalcError Requirements for This Story

`engine/error.rs` must define a `CalcError` enum usable by other stubs. Minimum required variants:
- `StackUnderflow` — pop/peek on empty stack
- `DivisionByZero` — divide or modulo by zero
- `DomainError(String)` — math domain error (e.g. sqrt of negative)
- `InvalidInput(String)` — unparseable user input
- `NotAnInteger` — bitwise op applied to float

Implement `std::fmt::Display` for `CalcError` so it can be converted to a user-visible message.
Do NOT derive `Serialize` or `Deserialize` on CalcError.

### Cargo.toml Changes

The existing `Cargo.toml` has 6 of the 8 required dependencies. Add:
```toml
toml = "~0.8"
signal-hook = "~0.3"
```

The `serde` entry must keep `features = ["derive"]` — it already does. No other changes needed.

### Existing File Considerations

- `engine/value.rs`: Has `unwrap_or(FBig::ZERO)` in `from_f64()` and string-based serialization. These are acceptable for now; Story 1.2 will replace with proper serde derives. If clippy flags `clippy::unwrap_used` or similar, suppress with `#[allow(clippy::unwrap_used)]` on the impl block — do NOT change the logic.
- `engine/stack.rs`: Has `StackEngine` struct — this is the wrong abstraction (target is CalcState), but leave it for Story 1.2 to replace. Add `#[allow(dead_code)]` if clippy warns about unused fields.
- `engine/ops.rs`: Uses string-based dispatch `apply_op(engine, "sin")` — this is the forbidden pattern per architecture, but it will be replaced in Story 1.4. Do NOT refactor.
- `engine/undo.rs`: Snapshots `Vec<CalcValue>` instead of full `CalcState` — will be replaced in Story 1.2.
- `input/parser.rs`: Returns `String` errors instead of `CalcError` — will be updated in Story 1.5. Already has tests that must continue to pass.

### Testing Standards

- Unit tests co-located in `#[cfg(test)]` blocks at the bottom of each source file
- Integration tests in `tests/integration/` only
- For this story: the only pre-existing tests are in `input/parser.rs`; they must continue to pass
- No new tests required in this story beyond the placeholder integration test files

### Quality Gate Commands

Run these in order; all must succeed before the story is done:
```bash
cargo build
cargo clippy -- -D warnings
cargo fmt
cargo test
```

### Project Structure Notes

- The `tests/integration/` files use Rust's integration test convention: they are separate binaries that `use rpncalc::...`. For placeholders, a simple `#[test] fn placeholder() {}` with no imports is sufficient and will compile.
- The `tui/widgets/` subdirectory requires a `widgets/mod.rs` — Rust module system requires a `mod.rs` (or `widgets.rs` at the parent level) for subdirectory modules. Use `mod.rs` inside the directory.
- Do not create `src/tui/widgets.rs` alongside `src/tui/widgets/` — only one form is valid.

### References

- Target module structure: [Source: architecture.md#Complete Project Directory Structure]
- Dependency versions: [Source: architecture.md#Core Dependencies]
- CalcError variants and naming: [Source: architecture.md#Naming Patterns]
- No unwrap() in non-test code rule: [Source: architecture.md#Process Patterns]
- Anti-patterns to avoid (string dispatch, etc.): [Source: architecture.md#Anti-patterns to reject in code review]
- Scaffold reorganization note: [Source: epics.md#Additional Requirements]
- Quality gates: [Source: architecture.md#Enforcement Guidelines]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- Fixed `impl IBig { fn is_zero() }` — cannot define inherent impl for external type; replaced with direct `*n == IBig::ZERO` comparisons
- Fixed `ops.rs:212` — `.pow(exp_u32)` expected `usize`, got `u32`; added `as usize` cast
- Fixed `clippy::module_inception` in `config/mod.rs` — added `#[allow(clippy::module_inception)]`
- Fixed `clippy::wrong_self_convention` in `angle.rs` — `from_radians(self, ...)` pattern; added allow attribute
- Fixed `clippy::unnecessary_lazy_evaluations` in `value.rs` — replaced `unwrap_or_else(|_| { f64::NAN })` with `unwrap_or(f64::NAN)`
- Fixed `tests/integration.rs` module path resolution — used `#[path = "integration/..."]` attributes since Cargo test binaries resolve modules relative to `tests/` not `tests/integration/`
- Added `#![allow(dead_code)]` at crate root to suppress 52 dead_code warnings from scaffold modules not yet wired up

### Senior Developer Review (AI)

**Outcome:** Approved with fixes applied
**Date:** 2026-03-18
**Action Items:** 3 total — 2 fixed, 1 deferred

#### Action Items
- [x] [Low] `format_fbig` visibility: changed `pub` → `pub(crate)` per architecture rule [src/engine/value.rs:142]
- [x] [Low] `#![allow(dead_code)]` breadth: added TODO comment clarifying it must be removed as modules wire up [src/main.rs:1]
- [x] [Low] Dead computation in `format_fbig` (`_sci` param never used): tracked for Story 1.2 overhaul [src/engine/value.rs:154]

### Completion Notes List

- All 8 required dependencies present in Cargo.toml (added `toml ~0.8` and `signal-hook ~0.3`)
- Created `engine/error.rs` with full CalcError enum (5 variants) and Display impl
- Created `engine/registers.rs` stub
- Updated `engine/mod.rs` with error + registers modules and `pub use error::CalcError` re-export
- Created complete `input/` module structure: mod.rs, action.rs, mode.rs, handler.rs, commands.rs
- Created complete `tui/` module structure: mod.rs, app.rs, layout.rs, widgets/{mod,stack_pane,input_line,hints_pane,error_line,mode_bar}.rs
- Created complete `config/` module structure: mod.rs, config.rs (with defaults), session.rs
- Created `tests/integration/` with placeholder files wired via `tests/integration.rs`
- All 4 pre-existing parser tests continue to pass
- 6 total tests pass (4 unit + 2 integration placeholders)
- `cargo build`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo test` all exit 0

### File List

Cargo.toml (modified)
src/main.rs (modified)
src/engine/mod.rs (modified)
src/engine/error.rs (created)
src/engine/registers.rs (created)
src/engine/angle.rs (modified — #[allow(clippy::wrong_self_convention)])
src/engine/value.rs (modified — removed illegal impl IBig, fixed clippy warnings)
src/engine/ops.rs (modified — fixed u32→usize cast)
src/config/mod.rs (created)
src/config/config.rs (created)
src/config/session.rs (created)
src/input/mod.rs (created)
src/input/action.rs (created)
src/input/mode.rs (created)
src/input/handler.rs (created)
src/input/commands.rs (created)
src/tui/mod.rs (created)
src/tui/app.rs (created)
src/tui/layout.rs (created)
src/tui/widgets/mod.rs (created)
src/tui/widgets/stack_pane.rs (created)
src/tui/widgets/input_line.rs (created)
src/tui/widgets/hints_pane.rs (created)
src/tui/widgets/error_line.rs (created)
src/tui/widgets/mode_bar.rs (created)
tests/integration.rs (created)
tests/integration/session_roundtrip.rs (created)
tests/integration/keybinding_sequences.rs (created)
