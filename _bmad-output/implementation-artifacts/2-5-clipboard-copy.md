# Story 2.5: Clipboard Copy

Status: done

## Story

As a CLI power user,
I want to copy the top stack value to my clipboard with a single keystroke,
So that I can paste results directly into other terminal applications without leaving the calculator.

## Acceptance Criteria

1. **Given** one or more values are on the stack, **When** the user presses `y`, **Then** the top stack value is copied to the system clipboard and the stack is unchanged.

2. **Given** DEC base is active, **When** the value is copied, **Then** it is copied as a plain decimal number (e.g. `42`, `3.14`).

3. **Given** HEX base is active with a specific style, **When** the value is copied, **Then** it is copied with that representation style (e.g. `0xFF`, `$FF`, `#FF`, or `FFh`).

4. **Given** the stack is empty, **When** the user presses `y`, **Then** an appropriate error is shown and the clipboard is unchanged.

## Tasks / Subtasks

- [x] Task 1: Implement `Action::Yank` handler in `src/tui/app.rs` (AC: 1–4)
  - [x] Add `use arboard::Clipboard;` import (arboard 3.x is already in Cargo.toml)
  - [x] Add `use crate::engine::base::{Base, HexStyle};` import to `app.rs`
  - [x] Add private `fn yank_text(val: &CalcValue, base: Base, hex_style: HexStyle) -> String` — see Dev Notes for full logic
  - [x] Replace the `Action::Yank` stub body with: peek at X register (do NOT pop), format via `yank_text`, write to clipboard via `arboard::Clipboard::new()?.set_text(text)`, set error on empty stack or clipboard failure, clear error on success

- [x] Task 2: Unit tests in `src/tui/app.rs` (AC: 1–4)
  - [x] `test_yank_empty_stack_sets_error` — empty stack → `error_message` is Some
  - [x] `test_yank_preserves_stack` — after Yank, stack depth unchanged
  - [x] `test_yank_text_dec` — `yank_text(Integer(42), Base::Dec, *)` → `"42"`
  - [x] `test_yank_text_hex_zerox` — `yank_text(Integer(255), Base::Hex, HexStyle::ZeroX)` → `"0xFF"`
  - [x] `test_yank_text_hex_dollar` — `yank_text(Integer(255), Base::Hex, HexStyle::Dollar)` → `"$FF"`
  - [x] `test_yank_text_hex_hash` — `yank_text(Integer(255), Base::Hex, HexStyle::Hash)` → `"#FF"`
  - [x] `test_yank_text_hex_suffix` — `yank_text(Integer(255), Base::Hex, HexStyle::Suffix)` → `"FFh"`
  - [x] `test_yank_text_float` — `yank_text(Float(3.14), Base::Dec, *)` → starts with `"3.14"`

- [x] Task 3: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — 204 tests passed (10 new)

## Dev Notes

### Files Changed — Exactly One

| File | Change |
|---|---|
| `src/tui/app.rs` | Implement `Action::Yank` + add `yank_text` helper + tests |

Do NOT touch `handler.rs` (already maps `y` → `Action::Yank`), `Cargo.toml` (arboard 3.x already present), `action.rs`, or any engine file.

### Wiring Already Done — Confirm Before Starting

- `handler.rs:26` — `KeyCode::Char('y') => Action::Yank` ✅ already wired
- `Cargo.toml` — `arboard = "3"` ✅ already present
- `action.rs:19` — `Yank` variant exists ✅ (marked `#[allow(dead_code)]` — that will go away once Yank has a real body)
- `app.rs` — Yank stub at the `Action::Yank` arm: `// Clipboard implementation deferred to Story 2.5`

### Action::Yank Handler — Required Imports

Add to `app.rs` imports:
```rust
use arboard::Clipboard;
use crate::engine::base::{Base, HexStyle};
use crate::engine::value::CalcValue;
```

`CalcValue` may already be used transitively — add explicitly if clippy complains.

### Action::Yank Handler — Implementation

```rust
Action::Yank => {
    match self.state.stack.last() {
        None => {
            self.error_message = Some("Stack is empty".into());
        }
        Some(val) => {
            let text = yank_text(val, self.state.base, self.state.hex_style);
            match Clipboard::new().and_then(|mut cb| cb.set_text(text)) {
                Ok(()) => {
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Clipboard error: {}", e));
                }
            }
        }
    }
}
```

Key: `stack.last()` peeks (does NOT pop) — stack is unchanged per AC 1.
`Clipboard::new()` can fail (no display server, Wayland/X11 issues) — must set error_message, never panic.
`and_then` chains the `set_text` call so both failure modes resolve to the same `Err` arm.

### yank_text Helper — Implementation

`display_with_base()` always uses `0x` prefix for HEX (e.g. `"0xFF"`). `HexStyle` must be applied as a post-processing step for non-ZeroX styles. Floats always display in decimal regardless of active base — HexStyle transformation only applies to `CalcValue::Integer` in HEX mode, but `yank_text` doesn't need to special-case this: the transformation operates on the string from `display_with_base()` and is safe because floats never produce a `"0x..."` string.

```rust
fn yank_text(val: &CalcValue, base: Base, hex_style: HexStyle) -> String {
    let raw = val.display_with_base(base);
    if base != Base::Hex || hex_style == HexStyle::ZeroX {
        return raw;
    }
    // raw is "0xABCD", "-0xABCD", or "0" (for the integer zero edge case)
    let negative = raw.starts_with('-');
    let sign = if negative { "-" } else { "" };
    let hex_part = if negative {
        raw.get(3..).unwrap_or(&raw) // strip "-0x"
    } else if raw.starts_with("0x") {
        raw.get(2..).unwrap_or(&raw) // strip "0x"
    } else {
        return raw; // edge case: integer zero displays as bare "0"
    };
    match hex_style {
        HexStyle::ZeroX => raw,  // unreachable — handled by early return above
        HexStyle::Dollar => format!("{}${}", sign, hex_part),
        HexStyle::Hash => format!("{}#{}", sign, hex_part),
        HexStyle::Suffix => format!("{}{}h", sign, hex_part),
    }
}
```

**Note:** `display_with_base(Base::Hex)` produces UPPERCASE hex digits (e.g. `"0xFF"`, not `"0xff"`). The `yank_text` output preserves this casing. Tests must use uppercase in expected values.

### Testing Yank — What Can and Cannot Be Tested

`arboard::Clipboard::new()` requires a display server (X11/Wayland). It will fail in headless CI. Therefore:

- **DO test:** `yank_text` formatting logic (pure function, no I/O)
- **DO test:** empty stack path (`test_yank_empty_stack_sets_error`)
- **DO test:** stack unchanged after Yank (`test_yank_preserves_stack`) — this tests the App logic without caring about clipboard success/failure
- **DO NOT attempt:** asserting `error_message.is_none()` after a full Yank in tests (clipboard write may fail in CI)

For `test_yank_preserves_stack`, push a value, call `app.apply(Action::Yank)`, assert `app.state.depth() == 1`. The error_message may be set (clipboard error in CI) but the stack must be untouched either way.

### Test Helper for yank_text

Since `yank_text` is a private function in the `app` module, tests in `#[cfg(test)] mod tests { use super::*; }` can call it directly:

```rust
#[test]
fn test_yank_text_hex_dollar() {
    let val = CalcValue::Integer(IBig::from(255));
    assert_eq!(
        yank_text(&val, Base::Hex, HexStyle::Dollar),
        "$FF"
    );
}
```

### display_with_base — HEX Output Reference

For `CalcValue::Integer(IBig::from(255))` with `Base::Hex`:
- `display_with_base(Base::Hex)` → `"0xFF"` (uppercase, ZeroX prefix)

After `yank_text` transformation:
- `HexStyle::ZeroX` → `"0xFF"`
- `HexStyle::Dollar` → `"$FF"`
- `HexStyle::Hash` → `"#FF"`
- `HexStyle::Suffix` → `"FFh"`

### Undo — NOT Required

`Action::Yank` does not mutate `CalcState`. Do not call `self.undo_history.snapshot()`. Undo/redo does not apply to clipboard operations.

### Error on Empty Stack

AC 4 requires an error when the stack is empty. Use:
```rust
self.error_message = Some("Stack is empty".into());
```
Do NOT copy anything to clipboard when the stack is empty.

### Architecture Compliance

- Architecture says `arboard` is the chosen clipboard crate (already in Cargo.toml)
- No `unwrap()` in non-test code — use `and_then` + match to handle all clipboard errors
- Error goes to `self.error_message` — App::apply() never returns Result
- Clipboard write is fire-and-forget: no confirmation feedback on success (UX principle: silence signals success)
- Stack is the truth — Yank never modifies it

### Previous Story Learnings (from 2.4)

- Keep imports tidy: `cargo fmt` may reformat grouped imports — let it
- The `action.rs` `#[allow(dead_code)]` on `Yank` should be removed once the handler is real (but clippy will tell you if it needs to stay or go)
- `app.rs` test pattern: `push_int(&mut app, n)` helper, then `app.apply(Action::*)`, assert state
- All new tests go in the existing `#[cfg(test)] mod tests { ... }` block at the bottom of `app.rs`
- Use `app.state.depth()` to check stack depth

### Project Structure

```
src/
└── tui/
    └── app.rs   ← only file to change
```

Do NOT create new files. Do NOT touch any other file.

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

### Completion Notes List

- Implemented `yank_text(val, base, hex_style) -> String` as a private module-level function in app.rs; accessible from `#[cfg(test)]` via `use super::*`
- `display_with_base(Base::Hex)` always produces `0x` prefix; `yank_text` post-processes for Dollar/Hash/Suffix styles
- Integer zero in HEX displays as bare `"0"` from `display_with_base` — `yank_text` returns it unchanged (no prefix to strip)
- Added two edge-case tests beyond spec: `test_yank_text_zero_hex` and `test_yank_text_negative_hex_dollar`
- `Clipboard::new()` gracefully handled via `and_then` — clipboard failures set `error_message`, never panic
- `cargo fmt` reformatted imports into multi-line grouped form and collapsed `Action::Yank => { match ... }` to `Action::Yank => match ...`
- 204 tests passing, all quality gates clean

### File List

- `src/tui/app.rs`
- `_bmad-output/implementation-artifacts/2-5-clipboard-copy.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
