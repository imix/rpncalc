# Story 3.3: Base & Angle Mode Switching

Status: done

## Story

As a CLI power user,
I want to switch the active numeric base and angle mode via chord keys,
So that I can work in hex, binary, or different angle units without leaving the calculator.

## Acceptance Criteria

1. **Given** the base chord is used (`x` then `h`), **When** HEX base is activated, **Then** all values currently on the stack redisplay in hexadecimal **And** the mode bar updates to show `HEX`.

2. **Given** any base (DEC, HEX, OCT, BIN) is activated via chord, **When** the stack renders, **Then** all values redisplay in the newly active base.

3. **Given** the angle mode chord is used (`m` then `r`), **When** RAD mode is activated, **Then** the mode bar updates to show `RAD` **And** subsequent trig operations use radians.

4. **Given** the hex style chord is used (`X` then a style key), **When** a hex style is selected (e.g. `0xFF`, `$FF`, `#FF`, `FFh`), **Then** hex values display with that prefix/suffix style **And** the mode bar updates to show the active style (only when HEX base is active).

## Tasks / Subtasks

- [x] Task 1: Add integration tests in `src/tui/app.rs` (AC: 1–4)
  - [x] `test_set_base_hex_updates_state` — `SetBase(Hex)` sets `state.base`, clears error
  - [x] `test_set_base_all_variants` — Dec/Hex/Oct/Bin all update `state.base` correctly
  - [x] `test_set_base_snapshots_undo` — `SetBase(Hex)` creates an undo snapshot
  - [x] `test_set_base_undo_restores_previous` — `SetBase(Hex)` then `Undo` → `state.base == Dec`
  - [x] `test_set_angle_mode_rad_updates_state` — `SetAngleMode(Rad)` sets `state.angle_mode`
  - [x] `test_set_angle_mode_affects_trig` — push 90.0, `SetAngleMode(Deg)` → `Sin` ≈ 1.0; push 90.0, `SetAngleMode(Rad)` → `Sin` ≠ 1.0 (proves angle mode gates trig)
  - [x] `test_set_hex_style_updates_state` — `SetHexStyle(Dollar)` sets `state.hex_style`
  - [x] `test_set_hex_style_all_variants` — all four HexStyle variants update state correctly
  - [x] `test_mode_changes_clear_error` — `SetBase` / `SetAngleMode` / `SetHexStyle` clear `error_message`

- [x] Task 2: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — 240 tests pass (231 pre-existing + 9 new)

## Dev Notes

### Zero New Production Code

All three actions are **fully implemented** end-to-end:

| Layer | Status |
|---|---|
| `app.rs::dispatch()` | `SetBase`, `SetAngleMode`, `SetHexStyle` all implemented — they mutate `state.base/angle_mode/hex_style` and return `Ok(())` |
| `handler.rs` (Story 3.2) | All chord second-key mappings wired: `x`+`h` → `SetBase(Hex)`, `m`+`r` → `SetAngleMode(Rad)`, etc. |
| `stack_pane.rs` | Calls `val.display_with_base(state.base)` — reactive, no changes needed |
| `mode_bar.rs` | Renders `state.angle_mode`, `state.base`, `state.hex_style` — reactive, no changes needed |

Story 3.3 is **integration test coverage only**. Do NOT touch production code.

### Dispatch Path for Mode Actions

Mode-change actions flow through the `action =>` fallthrough arm in `apply()` in `app.rs`:

```rust
action => {
    let was_chord = matches!(self.mode, AppMode::Chord(_));
    let pre_op = self.state.clone();   // snapshot BEFORE mutation
    match self.dispatch(action) {
        Ok(()) => {
            self.undo_history.snapshot(&pre_op);  // undo entry created on success
            self.error_message = None;
        }
        Err(e) => {
            self.error_message = Some(e.to_string());
        }
    }
    if was_chord { self.mode = AppMode::Normal; }  // always returns to Normal
}
```

Key implications:
- `SetBase` / `SetAngleMode` / `SetHexStyle` **always succeed** (return `Ok(())`) — they never produce an error
- Each creates an **undo snapshot** — mode changes are undoable
- `error_message` is cleared on every successful mode change
- `was_chord` flag returns to Normal mode — works correctly for chord-triggered mode changes

### Test Imports Needed

```rust
use crate::engine::{
    angle::AngleMode,
    base::{Base, HexStyle},
    ops::Op,
    value::CalcValue,
};
use crate::input::action::Action;
```

`CalcValue::from_f64(f: f64)` is available for pushing float values. Use it for the trig angle-mode test.

### Trig Angle Mode Test — Concrete Values

To prove `SetAngleMode` gates trig behavior without comparing floats exactly:

```rust
// DEG mode: sin(90.0°) = 1.0
// RAD mode: sin(90.0 rad) ≈ 0.894...  (sin of 90 radians, not degrees)
// These are definitively different — safe to assert != on their string representations
// OR compare as f64 with a tolerance
```

Simplest approach: use `state.angle_mode` equality to verify mode was set, then do a separate engine-level check OR just verify `state.angle_mode == AngleMode::Rad` after setting it, relying on the engine's angle conversion tests (already extensive in `ops.rs`) to prove trig uses the mode correctly.

### Previous Story Context (Story 3.2)

- Chord handler tests for base/angle/hex-style second-keys already exist in `handler.rs` — do NOT duplicate those; Story 3.3 tests App-level integration, not handler dispatch
- `was_chord` flag pattern is already tested for `Execute` (trig) actions — base/angle/hex tests should follow same pattern
- `test_chord_execute_returns_to_normal` and `test_chord_execute_failure_returns_to_normal` in `app.rs` are the template — new tests follow same structure
- 231 tests pass as baseline; new tests add to that count

### Do NOT Touch

- `src/input/handler.rs` — chord dispatch already complete, all second-key tests exist
- `src/tui/widgets/stack_pane.rs` — base display already tested (`test_value_uses_active_base`)
- `src/tui/widgets/mode_bar.rs` — all mode/base/hex_style display tests already exist (10 tests)
- `src/tui/widgets/hints_pane.rs` — chord submenu rendering complete
- Anything in `engine/` — angle mode and base display fully tested

### Architecture Compliance

- New tests are `#[cfg(test)]` at bottom of `app.rs` — no new files [Source: architecture.md#Test location]
- No `unwrap()` in non-test code [Source: architecture.md#No unwrap() policy]
- `CalcState` is the sole mutable state — tests verify `app.state.base/angle_mode/hex_style` [Source: architecture.md#Calculator State Ownership]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — clean implementation, no issues.

### Completion Notes List

- No production code changes required — all three actions (`SetBase`, `SetAngleMode`, `SetHexStyle`) were fully implemented in prior stories
- Added `AngleMode` to the test module imports in `app.rs`
- 9 new tests added to `src/tui/app.rs` covering all four ACs
- `test_set_angle_mode_affects_trig`: proves angle mode gates trig by comparing `sin(90 deg)` vs `sin(90 rad)` — results differ definitively
- `test_mode_changes_clear_error`: covers all three action types clearing `error_message` in a single test
- `cargo fmt` reformatted multi-arg assert macros to multi-line style

### File List

- `src/tui/app.rs` — added `AngleMode` import + 9 integration tests for SetBase/SetAngleMode/SetHexStyle
