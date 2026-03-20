# Story 4.2: Full-State Undo & Redo

Status: done

## Story

As a CLI power user,
I want to undo any operation and redo it if I change my mind,
So that I can experiment freely knowing every mistake is reversible.

## Acceptance Criteria

1. **Given** any state-mutating operation has been performed (stack change, register store/delete, mode change), **When** the user presses `u`, **Then** the complete calculator state is restored to exactly what it was before that operation — stack, registers, and active modes all revert.

2. **Given** an undo has been performed, **When** the user presses `Ctrl-R`, **Then** the undone operation is reapplied and the state advances forward again.

3. **Given** multiple operations have been performed, **When** the user presses `u` repeatedly, **Then** each press steps back through the operation history one at a time.

4. **Given** redo history exists, **When** the user performs any new state-mutating operation, **Then** the redo history is cleared — redo is only available for operations undone in the current chain.

5. **Given** there is nothing left to undo, **When** the user presses `u`, **Then** the stack is unchanged and `Nothing to undo` is shown.

6. **Given** there is nothing to redo, **When** the user presses `Ctrl-R`, **Then** the stack is unchanged and `Nothing to redo` is shown.

7. **Given** undo history has reached the configured depth limit, **When** a new operation is performed, **Then** the oldest undo snapshot is discarded to make room — the most recent history is always preserved.

## Tasks / Subtasks

- [x] Task 1: `src/tui/app.rs` — Wire `Config::max_undo_history` into `App::new()` (AC: 7)
  - [x] Add `use crate::config::config::Config;` import
  - [x] Replace `UndoHistory::new()` with `UndoHistory::with_max_depth(Config::load().max_undo_history)`

- [x] Task 2: `src/engine/undo.rs` — Remove stale `#[allow(dead_code)]` from `with_max_depth` (maintenance)
  - [x] Remove `#[allow(dead_code)]` attribute above `with_max_depth` — it is now called by `App::new()`

- [x] Task 3: Tests in `src/tui/app.rs` (AC: 1, 3, 4, 7)
  - [x] `test_undo_register_store` — StoreRegister snapshots; undo removes the register from state
  - [x] `test_undo_register_delete` — DeleteRegister snapshots; undo restores the deleted register
  - [x] `test_undo_restores_registers_and_stack_atomically` — push + store; undo; both stack depth and registers revert in one step
  - [x] `test_multiple_sequential_undos` — push three values, undo three times, stack empty, undo again → "Nothing to undo"
  - [x] `test_redo_after_multiple_undos` — push two values, undo twice, redo twice, depth = 2 again
  - [x] `test_depth_limit_discards_oldest` — set `app.undo_history = UndoHistory::with_max_depth(3)`; push 5 values; verify only 3 undos available

- [x] Task 4: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — 289 pre-existing + 6 new = 295 total

## Dev Notes

### What Is Already Fully Implemented (No Code Changes Needed)

**This story's engine is complete.** All mechanism work was done in earlier stories as a foundation:

| Component | Location | Status |
|-----------|----------|--------|
| `UndoHistory` struct | `src/engine/undo.rs` | ✅ Complete — `past: Vec<CalcState>`, `future: Vec<CalcState>`, depth limiting |
| `snapshot()` | `src/engine/undo.rs:28` | ✅ Complete — saves pre-op state, clears redo |
| `undo()` / `redo()` | `src/engine/undo.rs:37,47` | ✅ Complete — bidirectional stepping |
| `Action::Undo` / `Action::Redo` | `src/tui/app.rs:61,69` | ✅ Complete — "Nothing to undo/redo" messages |
| `'u'` → `Undo` binding | `src/input/handler.rs:30` | ✅ Complete |
| `Ctrl-R` → `Redo` binding | `src/input/handler.rs:17` | ✅ Complete |
| Snapshot before every op | `src/tui/app.rs:252-264` | ✅ Complete — the `action => { dispatch }` arm |
| Full `CalcState` snapshot | `src/engine/stack.rs` | ✅ Complete — `CalcState` includes stack, registers, angle_mode, base, hex_style |
| Depth limiting in UndoHistory | `src/engine/undo.rs:31-33` | ✅ Complete — removes oldest at `past[0]` |
| `with_max_depth()` constructor | `src/engine/undo.rs:19` | ✅ Exists but has `#[allow(dead_code)]` — needs Config wiring |

**Key insight:** `CalcState` is cloned whole on every snapshot. Undo restores the entire struct — stack, registers, and all modes — in a single atomic assignment. There is no partial restoration.

---

### Task 1: Config Wiring in `App::new()`

**Current `App::new()` (src/tui/app.rs:44-52):**

```rust
pub fn new() -> Self {
    Self {
        state: CalcState::new(),
        undo_history: UndoHistory::new(),   // ← hardcoded max_depth: 1000
        mode: AppMode::Normal,
        error_message: None,
        should_quit: false,
    }
}
```

**After change:**

```rust
use crate::config::config::Config;  // add to imports at top of file

pub fn new() -> Self {
    let config = Config::load();
    Self {
        state: CalcState::new(),
        undo_history: UndoHistory::with_max_depth(config.max_undo_history),
        mode: AppMode::Normal,
        error_message: None,
        should_quit: false,
    }
}
```

**Why this matters:** `Config::load()` currently returns `Config::default()` (Story 4.4 will add TOML loading). `Config::default().max_undo_history = 1000`. So behaviour is unchanged at runtime. The wiring is what matters — when Story 4.4 ships, the depth limit will automatically be honoured.

**`Config::load()` (src/config/config.rs:16-18):**

```rust
pub fn load() -> Self {
    Self::default()   // Story 4.4 will add TOML loading here
}
```

No changes to `config.rs` in this story.

---

### Task 2: Remove `#[allow(dead_code)]` from `with_max_depth`

In `src/engine/undo.rs`, line 18:

```rust
#[allow(dead_code)]          // ← remove this line
pub fn with_max_depth(max_depth: usize) -> Self {
```

Once Task 1 is done, `with_max_depth` is called from `App::new()` and the attribute is no longer needed. Clippy will enforce this if you run `cargo clippy -- -D warnings`.

Note: `can_undo` and `can_redo` also have `#[allow(dead_code)]` — leave those as-is; they are used in tests only and the attribute is appropriate.

---

### Task 3: Tests

All new tests go inside the existing `#[cfg(test)]` block in `src/tui/app.rs`, in the `// ── Story 4.2` section (after the existing 4.1 section).

**`test_undo_register_store`**

```rust
#[test]
fn test_undo_register_store() {
    let mut app = App::new();
    push_int(&mut app, 42);
    app.apply(Action::StoreRegister("x".into()));
    assert!(app.state.registers.contains_key("x"));
    app.apply(Action::Undo);
    // StoreRegister pops from stack and inserts to registers.
    // Undo restores the pre-op state: stack has 42 back, register is gone.
    assert_eq!(app.state.depth(), 1, "stack should be restored after undo");
    assert!(!app.state.registers.contains_key("x"), "register should be gone after undo");
}
```

**`test_undo_register_delete`**

```rust
#[test]
fn test_undo_register_delete() {
    let mut app = App::new();
    push_int(&mut app, 5);
    app.apply(Action::StoreRegister("tmp".into()));
    assert!(app.state.registers.contains_key("tmp"));
    app.apply(Action::DeleteRegister("tmp".into()));
    assert!(!app.state.registers.contains_key("tmp"));
    app.apply(Action::Undo);
    assert!(app.state.registers.contains_key("tmp"), "register should be restored after undo");
}
```

**`test_undo_restores_registers_and_stack_atomically`**

```rust
#[test]
fn test_undo_restores_registers_and_stack_atomically() {
    let mut app = App::new();
    push_int(&mut app, 10);
    push_int(&mut app, 20);
    // Store top value (20) into register — pops from stack
    app.apply(Action::StoreRegister("r".into()));
    assert_eq!(app.state.depth(), 1);
    assert!(app.state.registers.contains_key("r"));
    // Undo should restore: stack depth 2, register gone
    app.apply(Action::Undo);
    assert_eq!(app.state.depth(), 2, "both values on stack after undo");
    assert!(!app.state.registers.contains_key("r"), "register gone after undo");
}
```

**`test_multiple_sequential_undos`**

```rust
#[test]
fn test_multiple_sequential_undos() {
    let mut app = App::new();
    push_int(&mut app, 1);
    push_int(&mut app, 2);
    push_int(&mut app, 3);
    app.apply(Action::Undo);
    assert_eq!(app.state.depth(), 2);
    app.apply(Action::Undo);
    assert_eq!(app.state.depth(), 1);
    app.apply(Action::Undo);
    assert_eq!(app.state.depth(), 0);
    // One more undo — nothing left
    app.apply(Action::Undo);
    assert_eq!(app.error_message.as_deref(), Some("Nothing to undo"));
    assert_eq!(app.state.depth(), 0);
}
```

**`test_redo_after_multiple_undos`**

```rust
#[test]
fn test_redo_after_multiple_undos() {
    let mut app = App::new();
    push_int(&mut app, 10);
    push_int(&mut app, 20);
    app.apply(Action::Undo);
    app.apply(Action::Undo);
    assert_eq!(app.state.depth(), 0);
    app.apply(Action::Redo);
    assert_eq!(app.state.depth(), 1);
    app.apply(Action::Redo);
    assert_eq!(app.state.depth(), 2);
    // One more redo — nothing left
    app.apply(Action::Redo);
    assert_eq!(app.error_message.as_deref(), Some("Nothing to redo"));
}
```

**`test_depth_limit_discards_oldest`**

```rust
#[test]
fn test_depth_limit_discards_oldest() {
    use crate::engine::undo::UndoHistory;
    let mut app = App::new();
    app.undo_history = UndoHistory::with_max_depth(3);
    // Push 5 values — only last 3 snapshots should be retained
    for i in 1..=5 {
        push_int(&mut app, i);
    }
    assert_eq!(app.state.depth(), 5);
    // Can only undo 3 times (depth limit = 3)
    app.apply(Action::Undo);
    app.apply(Action::Undo);
    app.apply(Action::Undo);
    // Fourth undo should fail — oldest 2 snapshots were discarded
    app.apply(Action::Undo);
    assert_eq!(
        app.error_message.as_deref(),
        Some("Nothing to undo"),
        "oldest snapshots should have been discarded"
    );
}
```

---

### Undo Coverage Already Present (Do Not Duplicate)

The following AC scenarios are **already tested** in `src/tui/app.rs` from previous stories — do NOT add duplicate tests:

| AC | Existing test |
|----|--------------|
| AC1 stack restored | `test_undo_restores_state` |
| AC1 mode (base) restored | `test_set_base_undo_restores_previous` |
| AC2 redo after undo | `test_redo_after_undo` |
| AC4 new op clears redo | implicit in `test_redo_nothing_sets_error` |
| AC5 nothing to undo → message | `test_undo_nothing_sets_error` |
| AC6 nothing to redo → message | `test_redo_nothing_sets_error` |
| AC7 depth limit in UndoHistory | `engine::undo::tests::test_depth_limiting_discards_oldest` |

---

### Snapshot Verification: Which Operations Are Covered

All ops route through the `action => { ... }` arm in `App::apply()` which snapshots:

```rust
action => {
    let pre_op = self.state.clone();
    match self.dispatch(action) {
        Ok(()) => {
            self.undo_history.snapshot(&pre_op);  // snapshot on success only
            self.error_message = None;
        }
        Err(e) => {
            self.error_message = Some(e.to_string());  // no snapshot on failure
        }
    }
    ...
}
```

The `dispatch()` function covers: `Push`, `Execute`, `SetBase`, `SetAngleMode`, `SetHexStyle`, `StoreRegister`, `RecallRegister`, `DeleteRegister`.

Operations handled separately in `apply()` that also snapshot:
- `AlphaSubmit` (Alpha path): push/command dispatch — snapshots at lines 131-133, 138-141
- `AlphaSubmit` (AlphaStore path): register insert — snapshots at lines 117-120
- `AlphaSubmitThen`: push + op — two separate snapshots (lines 182-184, 207-209)

Operations that do NOT snapshot (by design — no state change):
- `Undo`, `Redo`, `Noop`, `Quit`, `AlphaChar`, `AlphaBackspace`, `AlphaCancel`, `EnterAlphaMode`, `EnterStoreMode`, `EnterChordMode`, `ChordCancel`, `ChordInvalid`, `Yank`

---

### Module Imports for `app.rs`

Current imports at top of `src/tui/app.rs`:

```rust
use crate::engine::{
    base::{Base, HexStyle},
    error::CalcError,
    ops,
    stack::CalcState,
    undo::UndoHistory,
    value::CalcValue,
};
use crate::input::{action::Action, commands::parse_command, mode::AppMode, parser::parse_value};
use arboard::Clipboard;
```

Add:

```rust
use crate::config::config::Config;
```

---

### Previous Story Learnings (Story 4.1)

- `cargo fmt` reformats code — run after all edits
- Test count was 289 after Story 4.1
- `App::new()` is public and works cleanly for test setup
- `pub` fields on `App` (`state`, `undo_history`, `mode`, `error_message`) are directly accessible in tests — use `app.undo_history = UndoHistory::with_max_depth(3)` instead of constructing a whole custom App
- `std::mem::replace` preferred over `.clone()` for mode transitions (not relevant here)
- Read each file before editing — exact match required by Edit tool

### Files to Change

| File | Change |
|------|--------|
| `src/tui/app.rs` | Add Config import; wire `Config::load().max_undo_history` into `App::new()`; add 6 tests |
| `src/engine/undo.rs` | Remove `#[allow(dead_code)]` from `with_max_depth` |

**No changes to:** `handler.rs`, `mode.rs`, `action.rs`, any widget, `config.rs`, engine files.

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — clean implementation.

### Completion Notes List

- Added `use crate::config::config::Config;` import to `tui/app.rs`
- `App::new()` now calls `Config::load()` and passes `config.max_undo_history` to `UndoHistory::with_max_depth()` — wires depth limit to config system; behaviour unchanged at runtime (default = 1000) until Story 4.4 adds TOML loading
- Removed `#[allow(dead_code)]` from `UndoHistory::with_max_depth` in `engine/undo.rs` — now actively used
- Added 6 new tests in `tui/app.rs` covering: register store undo, register delete undo, atomic register+stack restoration, multiple sequential undos, multi-step redo chain, depth limit at App level
- 295 total tests (289 pre-existing + 6 new); all pass; clippy clean

### File List

- `src/tui/app.rs` — Config import; `App::new()` wired to `Config::max_undo_history`; 6 new tests
- `src/engine/undo.rs` — removed `#[allow(dead_code)]` from `with_max_depth`

## Senior Developer Review

**Reviewer:** claude-sonnet-4-6
**Date:** 2026-03-19
**Outcome:** APPROVED — no findings

### AC Coverage

| AC | Tests | Result |
|----|-------|--------|
| AC1: undo restores stack+registers+modes | `test_undo_register_store`, `test_undo_register_delete`, `test_undo_restores_registers_and_stack_atomically`, pre-existing `test_set_base_undo_restores_previous` | ✅ |
| AC2: redo after undo | `test_redo_after_multiple_undos`, pre-existing `test_redo_after_undo` | ✅ |
| AC3: sequential undos step back | `test_multiple_sequential_undos` | ✅ |
| AC4: new op clears redo | `engine::undo::tests::test_new_action_after_undo_clears_redo` | ✅ |
| AC5: nothing to undo → message | `test_multiple_sequential_undos`, pre-existing `test_undo_nothing_sets_error` | ✅ |
| AC6: nothing to redo → message | `test_redo_after_multiple_undos`, pre-existing `test_redo_nothing_sets_error` | ✅ |
| AC7: depth limit wired to Config | `test_depth_limit_discards_oldest` + `App::new()` Config wiring | ✅ |

### Quality Gates

- `cargo build`: ✅ exits 0
- `cargo clippy -- -D warnings`: ✅ exits 0 (no output)
- `cargo test`: ✅ 295 passed, 0 failed
