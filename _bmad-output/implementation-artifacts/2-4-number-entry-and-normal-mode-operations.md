# Story 2.4: Number Entry & Normal Mode Operations

Status: done

## Story

As a CLI power user,
I want to type numbers naturally and execute operations with single keystrokes,
So that I can complete the 30-second session without thinking about the interface.

## Acceptance Criteria

1. **Given** the calculator is in normal mode, **When** the user presses any digit key (`0`–`9`), **Then** alpha (insert) mode is entered automatically and the digit appears in the input buffer.

2. **Given** the calculator is in normal mode, **When** the user presses `i`, **Then** alpha mode is entered and the input buffer is ready for text input (empty).

3. **Given** alpha mode is active with content in the buffer, **When** the user presses `Enter`, **Then** the value is pushed onto the stack (or command executed) and the calculator returns to normal mode.

4. **Given** alpha mode is active, **When** the user presses `Esc`, **Then** the buffer is cleared and the calculator returns to normal mode with the stack unchanged.

5. **Given** normal mode is active, **When** the user presses `Esc`, **Then** the calculator remains in normal mode — `Esc` is always safe to press.

6. **Given** alpha mode is active, **When** the user types characters, **Then** they appear in the input line with `> ` prefix and a `_` cursor at the end.

7. **Given** normal mode is active with sufficient stack values, **When** the user presses an operation key (`+`, `-`, `*`, `/`, `^`, `%`, `!`, `s`, `d`, `p`, `r`, `n`), **Then** the operation executes immediately with no additional keypress required and the stack updates.

8. **Given** normal mode is active with one or more values on the stack, **When** the user presses `Enter`, **Then** the top stack value is duplicated (HP convention).

## Tasks / Subtasks

- [x] Task 1: Add new `Action` variants in `src/input/action.rs` (AC: 1–4, 6)
  - [x] Add `AlphaChar(char)` — appends char to alpha buffer; if called in Normal mode, auto-enters alpha mode first
  - [x] Add `AlphaSubmit` — parse buffer and push/execute; return to Normal; if buffer empty, return to Normal silently
  - [x] Add `AlphaCancel` — clear buffer, return to Normal mode, clear error

- [x] Task 2: Implement `handle_key` in `src/input/handler.rs` (AC: 1–5, 7–8)
  - [x] **Normal mode** — wire ALL bindings below; replace the stub that only handles `q`:
    - [x] `'q'` → `Action::Quit` (already exists — keep)
    - [x] `'i'` → `Action::EnterAlphaMode`
    - [x] digit `'0'`–`'9'` → `Action::AlphaChar(c)` (auto-enters alpha, digit pre-seeded)
    - [x] `'+'` → `Action::Execute(Op::Add)`
    - [x] `'-'` → `Action::Execute(Op::Sub)`
    - [x] `'*'` → `Action::Execute(Op::Mul)`
    - [x] `'/'` → `Action::Execute(Op::Div)`
    - [x] `'^'` → `Action::Execute(Op::Pow)`
    - [x] `'%'` → `Action::Execute(Op::Mod)`
    - [x] `'!'` → `Action::Execute(Op::Factorial)`
    - [x] `'s'` → `Action::Execute(Op::Swap)`
    - [x] `'d'` → `Action::Execute(Op::Drop)`
    - [x] `'p'` → `Action::Execute(Op::Dup)`
    - [x] `'r'` → `Action::Execute(Op::Rotate)`
    - [x] `'n'` → `Action::Execute(Op::Negate)`
    - [x] `'u'` → `Action::Undo`
    - [x] `'y'` → `Action::Yank`
    - [x] `Ctrl+'r'` → `Action::Redo`
    - [x] `Enter` → `Action::Execute(Op::Dup)` (HP convention: dup top of stack)
    - [x] `Esc` → `Action::Noop` (AC 5: safe no-op in normal mode)
    - [x] everything else → `Action::Noop`
  - [x] **Alpha mode** — replace `_ => Action::Noop` stub:
    - [x] `Enter` → `Action::AlphaSubmit`
    - [x] `Esc` → `Action::AlphaCancel`
    - [x] any printable `KeyCode::Char(c)` → `Action::AlphaChar(c)`
    - [x] everything else → `Action::Noop`
  - [x] **Chord mode** — treat the same as Normal mode for now (`_ => Action::Noop`)

- [x] Task 3: Add action handlers in `src/tui/app.rs` (AC: 1–5)
  - [x] Add imports: `use crate::input::{commands::parse_command, parser::parse_value};`
  - [x] Add `Action::AlphaChar(c)` handler: if already in `AppMode::Alpha`, push `c` onto buffer; if in Normal/Chord, enter `AppMode::Alpha(c.to_string())` — always clear `error_message`
  - [x] Add `Action::AlphaSubmit` handler (see implementation detail in Dev Notes)
  - [x] Add `Action::AlphaCancel` handler: set `self.mode = AppMode::Normal`, clear `error_message`
  - [x] Add `Action::AlphaChar`, `Action::AlphaSubmit`, `Action::AlphaCancel` to the `unreachable!` exclusion list in `dispatch()`

- [x] Task 4: Implement `input_line::render` in `src/tui/widgets/input_line.rs` (AC: 6)
  - [x] Remove the `Block::bordered().title("Input")` stub — input line is a 1-row area with no border
  - [x] Normal mode → render `Paragraph::new("> ")` (prompt only, no cursor)
  - [x] Alpha mode → render `Paragraph::new(format!("> {}_", buf))` where `buf` is the buffer from `AppMode::Alpha(buf)`
  - [x] Chord mode → same as Normal (render `"> "`)

- [x] Task 5: Unit tests for `handle_key` in `src/input/handler.rs` (AC: 1–5, 7–8)
  - [x] `test_normal_digit_enters_alpha` — `'3'` in Normal → `AlphaChar('3')`
  - [x] `test_normal_i_enters_alpha_mode` — `'i'` in Normal → `EnterAlphaMode`
  - [x] `test_normal_esc_is_noop` — Esc in Normal → `Noop`
  - [x] `test_normal_enter_dups` — Enter in Normal → `Execute(Op::Dup)`
  - [x] `test_normal_ops` — test each of `+ - * / ^ % ! s d p r n` in Normal → correct `Execute(Op::*)`
  - [x] `test_normal_undo_redo` — `'u'` → `Undo`; `Ctrl+'r'` → `Redo`
  - [x] `test_alpha_char_appends` — printable char in Alpha → `AlphaChar(c)`
  - [x] `test_alpha_enter_submits` — Enter in Alpha → `AlphaSubmit`
  - [x] `test_alpha_esc_cancels` — Esc in Alpha → `AlphaCancel`

- [x] Task 6: Unit tests for new `App` action handlers in `src/tui/app.rs` (AC: 1–5)
  - [x] `test_alpha_char_in_normal_enters_alpha` — `AlphaChar('5')` in Normal → mode becomes `Alpha("5")`
  - [x] `test_alpha_char_appends_to_buffer` — `AlphaChar('1')` then `AlphaChar('2')` → buffer is `"12"`
  - [x] `test_alpha_submit_pushes_integer` — buffer `"42"`, `AlphaSubmit` → stack has 1 value, mode Normal
  - [x] `test_alpha_submit_pushes_float` — buffer `"3.14"`, `AlphaSubmit` → stack has 1 value, mode Normal
  - [x] `test_alpha_submit_empty_buffer_returns_to_normal` — empty buffer, `AlphaSubmit` → Normal, stack unchanged
  - [x] `test_alpha_submit_invalid_sets_error` — buffer `"garbage"`, `AlphaSubmit` → mode Normal, error_message set
  - [x] `test_alpha_cancel_returns_to_normal` — `AlphaCancel` in Alpha → mode Normal, stack unchanged

- [x] Task 7: Unit tests for `input_line::render` in `src/tui/widgets/input_line.rs` (AC: 6)
  - [x] `test_normal_mode_shows_prompt` — Normal → row contains `"> "`
  - [x] `test_alpha_mode_shows_buffer_and_cursor` — `Alpha("42")` → row contains `"> 42_"`
  - [x] `test_alpha_empty_buffer_shows_cursor` — `Alpha("")` → row contains `"> _"`

- [x] Task 8: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — 192 tests passed (169 existing + 23 new)

## Dev Notes

### Files Changed — Exactly Four

| File | Change |
|---|---|
| `src/input/action.rs` | Add 3 new variants |
| `src/input/handler.rs` | Full implementation (replaces 3-line stub) |
| `src/tui/app.rs` | Add 3 action handlers + parser imports |
| `src/tui/widgets/input_line.rs` | Replace stub |

Do NOT touch `layout.rs`, `main.rs`, engine files, `stack_pane.rs`, `mode_bar.rs`, `error_line.rs`.

### New Action Variants

```rust
// Add to the Action enum in action.rs:
AlphaChar(char),   // type one character into the alpha buffer
AlphaSubmit,       // commit the alpha buffer (Enter key in alpha mode)
AlphaCancel,       // discard the alpha buffer (Esc key in alpha mode)
```

Also add `AlphaChar(char)`, `AlphaSubmit`, `AlphaCancel` to the `unreachable!()` list in `app.rs::dispatch()`:
```rust
Action::Quit
| Action::Noop
| Action::Undo
| Action::Redo
| Action::EnterAlphaMode
| Action::Yank
| Action::AlphaChar(_)
| Action::AlphaSubmit
| Action::AlphaCancel => unreachable!("handled in apply()"),
```

### handler.rs — Required Imports

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::engine::ops::Op;
use crate::input::{action::Action, mode::AppMode};
```

`KeyModifiers` is needed for `Ctrl-R` (redo). Check it as:
```rust
KeyCode::Char('r') if event.modifiers.contains(KeyModifiers::CONTROL) => Action::Redo,
```

### handler.rs — Pattern Order Matters

In Rust match arms, order matters. For Normal mode, the digit guard must come BEFORE any catch-all:
```rust
AppMode::Normal => match event.code {
    KeyCode::Char('q') => Action::Quit,
    KeyCode::Char('i') => Action::EnterAlphaMode,
    KeyCode::Char(c) if c.is_ascii_digit() => Action::AlphaChar(c),
    KeyCode::Char('r') if event.modifiers.contains(KeyModifiers::CONTROL) => Action::Redo,
    KeyCode::Char('+') => Action::Execute(Op::Add),
    // ... all ops ...
    KeyCode::Enter => Action::Execute(Op::Dup),
    KeyCode::Esc => Action::Noop,
    _ => Action::Noop,
},
```

**CRITICAL:** `Ctrl-R` must be checked BEFORE the regular `'r'` → `Rotate` binding, otherwise the guard check fires first. In the pattern above: `Char('r') if modifiers.CONTROL` is before `Char('r')` (which would map to Rotate). Alternatively:
```rust
KeyCode::Char('r') => {
    if event.modifiers.contains(KeyModifiers::CONTROL) {
        Action::Redo
    } else {
        Action::Execute(Op::Rotate)
    }
}
```

### app.rs — AlphaChar Handler

```rust
Action::AlphaChar(c) => {
    if let AppMode::Alpha(ref mut buf) = self.mode {
        buf.push(c);
    } else {
        self.mode = AppMode::Alpha(c.to_string());
    }
    self.error_message = None;
}
```

### app.rs — AlphaSubmit Handler

This is the most complex handler. Requires importing `parse_value` and `parse_command`:

```rust
use crate::input::{commands::parse_command, parser::parse_value};
```

Handler logic:
```rust
Action::AlphaSubmit => {
    let buf = match &self.mode {
        AppMode::Alpha(buf) => buf.clone(),
        _ => return, // shouldn't be dispatched from non-alpha mode
    };
    self.mode = AppMode::Normal;
    self.error_message = None;

    if buf.is_empty() {
        // Empty buffer: return to Normal silently (no dup — HP dup is only from Normal mode Enter)
        return;
    }

    // Try parsing as a value first
    if let Ok(val) = parse_value(&buf) {
        let pre_op = self.state.clone();
        self.state.push(val);
        self.undo_history.snapshot(&pre_op);
        return;
    }

    // Try parsing as a command (e.g., "myvar RCL", "myvar STORE")
    match parse_command(&buf) {
        Ok(action) => {
            let pre_op = self.state.clone();
            match self.dispatch(action) {
                Ok(()) => {
                    self.undo_history.snapshot(&pre_op);
                }
                Err(e) => {
                    self.error_message = Some(e.to_string());
                }
            }
        }
        Err(_) => {
            self.error_message = Some(format!("Unknown input: {}", buf));
        }
    }
}
```

### app.rs — AlphaCancel Handler

```rust
Action::AlphaCancel => {
    self.mode = AppMode::Normal;
    self.error_message = None;
}
```

### input_line.rs — Implementation

This is a 1-row widget (like mode_bar and error_line). **No border.** Required imports:

```rust
use ratatui::{
    layout::Rect,
    widgets::Paragraph,
    Frame,
};
use crate::input::mode::AppMode;
```

Render:
```rust
pub fn render(f: &mut Frame, area: Rect, mode: &AppMode) {
    let text = match mode {
        AppMode::Alpha(buf) => format!("> {}_", buf),
        _ => "> ".to_string(),
    };
    f.render_widget(Paragraph::new(text), area);
}
```

### Testing handle_key — Key Event Construction

crossterm `KeyEvent` constructor in tests:
```rust
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn ctrl_key(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}
```

Usage in tests:
```rust
assert_eq!(handle_key(&AppMode::Normal, key(KeyCode::Char('3'))), Action::AlphaChar('3'));
assert_eq!(handle_key(&AppMode::Normal, ctrl_key('r')), Action::Redo);
assert_eq!(handle_key(&AppMode::Alpha("".into()), key(KeyCode::Enter)), Action::AlphaSubmit);
```

### Testing App — AlphaChar / AlphaSubmit / AlphaCancel

Use the existing `App::new()` + `App::apply()` pattern from existing tests. To set alpha mode state, chain actions:
```rust
// Enter alpha mode with '4' pre-seeded
let mut app = App::new();
app.apply(Action::AlphaChar('4'));
assert_eq!(app.mode, AppMode::Alpha("4".to_string()));

// Submit
app.apply(Action::AlphaChar('2'));
app.apply(Action::AlphaSubmit);
assert_eq!(app.state.depth(), 1);
assert_eq!(app.mode, AppMode::Normal);
```

### Testing input_line — TestBackend Pattern (same as Story 2.2/2.3)

Use `TestBackend::new(width, 1)` with height=1 (no border). Row to inspect is row 0.

```rust
fn render_input_line(mode: &AppMode, width: u16) -> ratatui::buffer::Buffer {
    let backend = TestBackend::new(width, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| render(f, f.area(), mode)).unwrap();
    terminal.backend().buffer().clone()
}
```

### Previous Story Learnings

- **1-row widgets use no border**: `Paragraph::new(text)` directly into `area`. Do NOT use `Block::bordered()` (wastes 2 rows). All three of `error_line`, `mode_bar`, `input_line` are borderless 1-row strips.
- **`buf.cell((x, y)).unwrap()`** is the non-deprecated API for TestBackend (not `buf.get(x, y)`).
- **`parse_value` and `parse_command` are `#[allow(dead_code)]`** in their current files — remove or leave the attribute (removing is cleaner). The `#[allow(dead_code)]` on `pub fn parse_value` and `pub fn parse_command` can be dropped since they'll now be used by `app.rs`.
- **`app.rs` is the most tested file** — 17 existing tests. Don't break them. All new action handlers sit alongside existing handlers in `apply()`. The `unreachable!()` arm must be kept current.
- **`dispatch()` must list new actions** in the unreachable arm, otherwise clippy will warn about non-exhaustive match if new variants are not handled.

### Operation Key → Op Mapping (AC 7)

| Key | Op | Notes |
|---|---|---|
| `+` | `Op::Add` | binary |
| `-` | `Op::Sub` | binary |
| `*` | `Op::Mul` | binary |
| `/` | `Op::Div` | binary |
| `^` | `Op::Pow` | binary |
| `%` | `Op::Mod` | binary |
| `!` | `Op::Factorial` | unary |
| `s` | `Op::Swap` | stack |
| `d` | `Op::Drop` | stack |
| `p` | `Op::Dup` | stack |
| `r` | `Op::Rotate` | stack (regular `r`, not Ctrl-R) |
| `n` | `Op::Negate` | unary |

Additional normal mode bindings (from UX design):
- `u` → `Action::Undo`
- `Ctrl-R` → `Action::Redo`
- `y` → `Action::Yank` (clipboard, already in action.rs; Story 2.5 completes it)
- `Enter` → `Action::Execute(Op::Dup)` (HP convention, AC 8)
- `Esc` → `Action::Noop` (AC 5)

### Alpha Mode Buffer Rules

- **Any `KeyCode::Char(c)` in Alpha mode → `AlphaChar(c)`** regardless of what `c` is. The user can type any character (digits, letters, `.`, `-`, `_`, space, etc.). The parser handles validation at submit time.
- **Only `Enter` and `Esc` have special meaning in Alpha mode.** All other named keys (arrows, F-keys, backspace, etc.) → `Noop` for now.
- **Buffer is stored in `AppMode::Alpha(String)`** — the buffer is the single source of truth.

### Architecture Compliance

- `handle_key` is a pure function: `(mode, event) → action`. No side effects. Fully testable without TUI.
- `app.rs::apply()` owns all state mutation. Handler only translates keys to actions.
- `parse_value` and `parse_command` are the parsing entry points — use them directly, do not reimplement parsing logic in `app.rs`.
- `undo_history.snapshot()` is called BEFORE the operation for push (pre-op snapshot), so undo restores the pre-push state. Maintain this pattern.

### Project Structure

```
src/
├── input/
│   ├── action.rs     ← ✅ add AlphaChar, AlphaSubmit, AlphaCancel
│   ├── handler.rs    ← ✅ full implementation
│   ├── commands.rs   ← DO NOT TOUCH (already complete)
│   ├── parser.rs     ← DO NOT TOUCH (already complete)
│   └── mode.rs       ← DO NOT TOUCH
└── tui/
    ├── app.rs        ← ✅ add 3 handlers + imports
    └── widgets/
        ├── input_line.rs  ← ✅ replace stub
        ├── mode_bar.rs    ← DO NOT TOUCH (Story 2.3, done)
        ├── error_line.rs  ← DO NOT TOUCH (Story 2.3, done)
        └── stack_pane.rs  ← DO NOT TOUCH (Story 2.2, done)
```

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

### Completion Notes List

- All 8 tasks completed. 194 tests passing (25 new tests added after code review fixes).
- `cargo fmt` reformatted handler.rs test imports and a few assert_eq! calls for line-length compliance.
- Handler tests (Task 5) were co-located with implementation in handler.rs as planned.
- `AlphaChar` in Normal mode auto-transitions to `AppMode::Alpha(c.to_string())` per AC 1.
- Ctrl-R (Redo) correctly placed before plain `'r'` (Rotate) arm in match to ensure correct dispatch.
- Code review (2026-03-19): Fixed M1 — added `test_alpha_submit_store_command` and `test_alpha_submit_recall_command` to cover the command dispatch path in AlphaSubmit. Fixed L1 — narrowed blanket `#[allow(dead_code)]` on Action enum to per-variant suppressors with rationale comments for each deferred variant.
- Post-retro bug fixes (2026-03-19): Three behaviors discovered missing during manual testing and corrected:
  1. **Backspace in Alpha mode** — `KeyCode::Backspace` now dispatches `Action::AlphaBackspace`; pops last char from buffer, returns to Normal if buffer becomes empty.
  2. **Operator auto-submit from Alpha mode** — Pressing an op key (`+`, `-`, `*`, `/`, `^`, `%`, `!`, `n`, `s`, `d`, `p`, `r`) while in Alpha mode now dispatches `Action::AlphaSubmitThen(op)`: submits buffer as value first, then executes op. Enables natural `4 ENTER 4 +` flow.
  3. **Stack labels corrected to HP48 convention** — Labels changed from `X:/Y:/Z:/T:` to `1:/2:/3:/4:` from bottom (see story 2.2 and planning docs). Corrected in stack_pane.rs.

### File List

- `src/input/action.rs`
- `src/input/handler.rs`
- `src/tui/app.rs`
- `src/tui/widgets/input_line.rs`
- `_bmad-output/implementation-artifacts/2-4-number-entry-and-normal-mode-operations.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
