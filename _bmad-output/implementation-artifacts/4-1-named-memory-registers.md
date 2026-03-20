# Story 4.1: Named Memory Registers

Status: done

## Story

As a power user,
I want to store values in named registers with a dedicated shortcut and see all register operations in the hints pane,
So that I can save and recall intermediate results without memorizing the STORE/RCL command syntax.

## Acceptance Criteria

1. **Given** normal mode with a value on the stack, **When** the user presses `S`, **Then** the app enters a store-name input mode (AlphaStore) prompting the user to type a register name.

2. **Given** AlphaStore mode, **When** the user types a name and presses Enter, **Then** the top-of-stack value is stored in that register and the stack is unchanged; the hints pane register section updates to show the new register.

3. **Given** normal mode, **When** the user types `myvar DEL` and presses Enter, **Then** the named register `myvar` is deleted; if the register does not exist, a clear error is shown.

4. **Given** alpha mode (number entry) with a number in the buffer, **When** the user presses `+`, `-`, `*`, `/`, `^`, `%`, `!`, `n`, `s`, `d`, `p`, or `r`, **Then** the buffer is pushed and the corresponding operation executes in one keystroke — these bindings are now shown in the hints pane alpha state.

5. **Given** the hints pane is visible in normal mode, **When** there are no registers, **Then** the STACK section shows `S  store` as a discoverable hint.

6. **Given** the hints pane is visible in normal mode, **When** the user is in AlphaStore mode, **Then** the hints pane shows "Enter store / Esc cancel / Bksp delete" (store-specific prompt).

7. **Given** the user types a non-numeric, non-command string (e.g., `hello`) and presses Enter, **Then** the error message reads: `Unknown input: hello (use 'name STORE' or 'name RCL')`.

8. **Given** normal mode with an empty stack, **When** the user presses `S`, **Then** an error is shown: `Cannot store: stack is empty` and the mode remains Normal.

## Tasks / Subtasks

- [x] Task 1: `src/input/commands.rs` — add DEL command (AC: 3)
  - [x] Add `[name, "DEL"] => Ok(Action::DeleteRegister(name.to_string()))` branch

- [x] Task 2: `src/input/mode.rs` — add AlphaStore variant (AC: 1, 2, 6)
  - [x] Add `AlphaStore(String)` variant to `AppMode` enum

- [x] Task 3: `src/input/action.rs` — add EnterStoreMode action (AC: 1, 8)
  - [x] Add `EnterStoreMode` variant to `Action` enum

- [x] Task 4: `src/input/handler.rs` — wire S key and AlphaStore key handling (AC: 1, 2, 6)
  - [x] In normal mode arm: add `'S' => Action::EnterStoreMode`
  - [x] Add `AppMode::AlphaStore(_)` arm with Enter→AlphaSubmit, Esc→AlphaCancel, Backspace→AlphaBackspace, Char(c)→AlphaChar(c)

- [x] Task 5: `src/tui/app.rs` — handle EnterStoreMode and AlphaStore in dispatch/apply (AC: 1, 2, 8)
  - [x] Handle `Action::EnterStoreMode` in `apply()`: empty stack → error; else → `self.mode = AppMode::AlphaStore(String::new())`
  - [x] Update `AlphaChar` handler: match both `AppMode::Alpha` and `AppMode::AlphaStore`
  - [x] Update `AlphaBackspace` handler: match both `AppMode::Alpha` and `AppMode::AlphaStore`
  - [x] Update `AlphaCancel` handler: match both `AppMode::Alpha` and `AppMode::AlphaStore` (already unconditional — no code change needed)
  - [x] Update `AlphaSubmit` handler: branch on `AppMode::AlphaStore` → peek top value → insert to registers (stack unchanged)
  - [x] Improve error message in AlphaSubmit/parse fallback: `Unknown input: {} (use 'name STORE' or 'name RCL')`
  - [x] Add `EnterStoreMode` to `dispatch()` unreachable arm

- [x] Task 6: `src/tui/widgets/mode_bar.rs` — handle AlphaStore (AC: 6)
  - [x] Add `AppMode::AlphaStore(_) => "[INSERT]"` to the mode label match arm

- [x] Task 7: `src/tui/widgets/input_line.rs` — handle AlphaStore (AC: 6)
  - [x] Add `AppMode::AlphaStore(buf) => format!("> {}_", buf)` to the render match arm

- [x] Task 8: `src/tui/widgets/hints_pane.rs` — enrich all three mode states (AC: 4, 5, 6)
  - [x] Add `("S", "store")` to `STACK_OPS` so it appears in normal mode
  - [x] Enrich Alpha state: add 5 rows of AlphaSubmitThen bindings below the 3 existing lines:
    - `+  add   -  sub`
    - `*  mul   /  div`
    - `^  pow   !  fact`
    - `n  neg   s  swap`
    - `d  drop  r  rot`
  - [x] Add AlphaStore state render branch: show "Enter store / Esc cancel / Bksp delete"

- [x] Task 9: Tests
  - [x] `commands.rs`: test `parse_command("myvar DEL")` returns `DeleteRegister("myvar")`
  - [x] `app.rs`: test `EnterStoreMode` with empty stack → error, mode stays Normal
  - [x] `app.rs`: test `EnterStoreMode` with value on stack → mode becomes `AlphaStore("")`
  - [x] `app.rs`: test AlphaChar/AlphaBackspace/AlphaCancel work in `AlphaStore` mode
  - [x] `app.rs`: test AlphaSubmit in AlphaStore mode → register stored, mode → Normal
  - [x] `app.rs`: test `AlphaSubmit` fallback error message contains "(use 'name STORE' or 'name RCL')"
  - [x] `hints_pane.rs`: test `("S", "store")` appears in normal mode render (depth ≥ 0)
  - [x] `hints_pane.rs`: test alpha mode render contains `+  add` (AlphaSubmitThen bindings visible)
  - [x] `hints_pane.rs`: test AlphaStore render shows "Enter store"
  - [x] `mode_bar.rs`: test `AlphaStore("")` renders `[INSERT]`
  - [x] `input_line.rs`: test `AlphaStore("my")` renders `> my_`

- [x] Task 10: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — 261 pre-existing + 28 new = 289 total

## Dev Notes

### Files to Change

| File | Change |
|------|--------|
| `src/input/commands.rs` | Add DEL branch |
| `src/input/mode.rs` | Add `AlphaStore(String)` variant |
| `src/input/action.rs` | Add `EnterStoreMode` variant |
| `src/input/handler.rs` | Wire `S` key; add AlphaStore arm |
| `src/tui/app.rs` | Handle `EnterStoreMode`; update Alpha/AlphaStore shared handlers |
| `src/tui/widgets/mode_bar.rs` | AlphaStore → `[INSERT]` |
| `src/tui/widgets/input_line.rs` | AlphaStore → `> buf_` |
| `src/tui/widgets/hints_pane.rs` | Enrich Alpha; add AlphaStore; add S to normal |

No new files. No changes to engine files.

---

### Task 1: `src/input/commands.rs` — DEL Command

Current state (relevant section):

```rust
pub fn parse_command(input: &str) -> Result<Action, String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    match parts.as_slice() {
        [name, "STORE"] => Ok(Action::StoreRegister(name.to_string())),
        [name, "RCL"] => Ok(Action::RecallRegister(name.to_string())),
        // ... other arms
    }
}
```

Add immediately after the `RCL` arm:

```rust
[name, "DEL"] => Ok(Action::DeleteRegister(name.to_string())),
```

---

### Task 2: `src/input/mode.rs` — AlphaStore Variant

Current `AppMode`:

```rust
pub enum AppMode {
    Normal,
    Alpha(String),
    Chord(ChordCategory),
}
```

After change:

```rust
pub enum AppMode {
    Normal,
    Alpha(String),
    AlphaStore(String),
    Chord(ChordCategory),
}
```

`AlphaStore` holds the register name being typed, exactly parallel to `Alpha` which holds the number being typed.

---

### Task 3: `src/input/action.rs` — EnterStoreMode

Current actions include `StoreRegister(String)`, `RecallRegister(String)`, `DeleteRegister(String)`. Add:

```rust
EnterStoreMode,
```

Place it near the other register-related actions for readability.

---

### Task 4: `src/input/handler.rs` — Key Wiring

#### Normal mode: add `S` binding

In `handle_key` under `AppMode::Normal`, the key match currently has entries like `'t' => ...`, `'l' => ...`, etc. Add:

```rust
'S' => Action::EnterStoreMode,
```

`S` (uppercase) is currently unbound in normal mode. Confirmed free in handler.rs.

#### AlphaStore arm

Add a new match arm for `AppMode::AlphaStore(_)` that mirrors `AppMode::Alpha(_)` exactly:

```rust
AppMode::AlphaStore(_) => match key.code {
    KeyCode::Enter => Action::AlphaSubmit,
    KeyCode::Esc => Action::AlphaCancel,
    KeyCode::Backspace => Action::AlphaBackspace,
    KeyCode::Char(c) => Action::AlphaChar(c),
    _ => Action::Noop,
},
```

---

### Task 5: `src/tui/app.rs` — dispatch/apply Changes

#### 5a: Handle `EnterStoreMode` in `apply()`

`apply()` handles mode transitions. Add:

```rust
Action::EnterStoreMode => {
    if self.state.stack.is_empty() {
        self.error_message = Some("Cannot store: stack is empty".to_string());
    } else {
        self.mode = AppMode::AlphaStore(String::new());
    }
}
```

#### 5b: `AlphaChar` — support both Alpha and AlphaStore

Current:
```rust
Action::AlphaChar(c) => {
    if let AppMode::Alpha(ref mut buf) = self.mode {
        buf.push(c);
    }
}
```

After:
```rust
Action::AlphaChar(c) => {
    match self.mode {
        AppMode::Alpha(ref mut buf) | AppMode::AlphaStore(ref mut buf) => {
            buf.push(c);
        }
        _ => {}
    }
}
```

#### 5c: `AlphaBackspace` — support both Alpha and AlphaStore

Same pattern as AlphaChar:

```rust
Action::AlphaBackspace => {
    match self.mode {
        AppMode::Alpha(ref mut buf) | AppMode::AlphaStore(ref mut buf) => {
            buf.pop();
        }
        _ => {}
    }
}
```

#### 5d: `AlphaCancel` — support both Alpha and AlphaStore

Current:
```rust
Action::AlphaCancel => {
    if matches!(self.mode, AppMode::Alpha(_)) {
        self.mode = AppMode::Normal;
    }
}
```

After:
```rust
Action::AlphaCancel => {
    if matches!(self.mode, AppMode::Alpha(_) | AppMode::AlphaStore(_)) {
        self.mode = AppMode::Normal;
    }
}
```

#### 5e: `AlphaSubmit` — branch on AlphaStore vs Alpha

Current `AlphaSubmit` logic: take buffer, try `parse_value`, then `parse_command`, then error.

New logic:

```rust
Action::AlphaSubmit => {
    // Check if we're in store mode — if so, buffer is a register name
    if let AppMode::AlphaStore(buf) = &self.mode.clone() {
        let name = buf.trim().to_string();
        self.mode = AppMode::Normal;
        if name.is_empty() {
            self.error_message = Some("Register name cannot be empty".to_string());
        } else {
            self.dispatch(Action::StoreRegister(name));
        }
        return;
    }

    // Normal Alpha mode: buffer is a number or command
    if let AppMode::Alpha(buf) = &self.mode.clone() {
        let input = buf.trim().to_string();
        self.mode = AppMode::Normal;
        if input.is_empty() {
            return;
        }
        match parse_value(&input) {
            Ok(val) => self.dispatch(Action::Push(val)),
            Err(_) => match parse_command(&input) {
                Ok(action) => self.dispatch(action),
                Err(_) => {
                    self.error_message = Some(format!(
                        "Unknown input: {} (use 'name STORE' or 'name RCL')",
                        input
                    ));
                }
            },
        }
    }
}
```

**Note on `.clone()`**: The borrow checker requires cloning `self.mode` before mutating `self.mode` in the same block. Pattern: `let mode = self.mode.clone(); self.mode = AppMode::Normal; match mode { ... }`.

Alternative (cleaner):

```rust
Action::AlphaSubmit => {
    let mode = std::mem::replace(&mut self.mode, AppMode::Normal);
    match mode {
        AppMode::AlphaStore(buf) => {
            let name = buf.trim().to_string();
            if name.is_empty() {
                self.error_message = Some("Register name cannot be empty".to_string());
            } else {
                self.dispatch(Action::StoreRegister(name));
            }
        }
        AppMode::Alpha(buf) => {
            let input = buf.trim().to_string();
            if input.is_empty() {
                return;
            }
            match parse_value(&input) {
                Ok(val) => self.dispatch(Action::Push(val)),
                Err(_) => match parse_command(&input) {
                    Ok(action) => self.dispatch(action),
                    Err(_) => {
                        self.error_message = Some(format!(
                            "Unknown input: {} (use 'name STORE' or 'name RCL')",
                            input
                        ));
                    }
                },
            }
        }
        _ => {}
    }
}
```

Use `std::mem::replace` — it avoids clone and is idiomatic Rust for this pattern.

#### 5f: Add `EnterStoreMode` to `dispatch()` unreachable list

`dispatch()` handles engine-level actions. `EnterStoreMode` is a UI-level action handled by `apply()`. Add it to the `unreachable!()` arm or the catch-all comment section — whichever pattern the existing code uses for actions that apply() handles.

---

### Task 6: `src/tui/widgets/mode_bar.rs`

Current match for mode label:

```rust
let mode_label = match &app_mode {
    AppMode::Normal => "NORMAL",
    AppMode::Alpha(_) => "INSERT",
    AppMode::Chord(_) => "CHORD",
};
```

Wait — the actual text shown is `[NORMAL]` / `[INSERT]` / `[CHORD]`. Add:

```rust
AppMode::AlphaStore(_) => "INSERT",
```

The INSERT label is correct — user is typing text (a name). The hints pane provides the context that it's a store operation.

---

### Task 7: `src/tui/widgets/input_line.rs`

Current:

```rust
let text = match &mode {
    AppMode::Normal => String::new(),
    AppMode::Alpha(buf) => format!("> {}_", buf),
    AppMode::Chord(_) => String::new(),
};
```

Add:

```rust
AppMode::AlphaStore(buf) => format!("> {}_", buf),
```

Same display format as Alpha — user sees `> myvar_` while typing the register name.

---

### Task 8: `src/tui/widgets/hints_pane.rs`

#### 8a: Add `S` to STACK_OPS in normal mode

`STACK_OPS` is a `&[(&str, &str)]` constant. Currently:

```rust
const STACK_OPS: &[(&str, &str)] = &[
    ("s", "swap"), ("d", "drop"), ("p", "pop"),
    ("r", "rot"),  ("u", "undo"), ("y", "redo"),
];
```

Add `("S", "store")` — placement: after `("y", "redo")` or as first entry (most discoverable position):

```rust
const STACK_OPS: &[(&str, &str)] = &[
    ("s", "swap"), ("d", "drop"), ("p", "pop"),
    ("r", "rot"),  ("u", "undo"), ("y", "redo"),
    ("S", "store"),
];
```

This gives 7 entries. The grid renders 2 per row → 4 rows (last row has 1 item). Still fits.

#### 8b: Enrich Alpha mode hints

Current alpha section (inside `render` for `AppMode::Alpha(_)`):

```rust
lines.push(Line::from(vec![
    Span::styled("Enter", key_style), Span::raw(" push  "),
    Span::styled("Esc",   key_style), Span::raw(" cancel"),
]));
lines.push(Line::from(vec![
    Span::styled("Bksp",  key_style), Span::raw(" delete"),
]));
```

Wait — the actual current implementation may be simpler static text. Check the exact current format then add below those 3 lines (or however many exist):

Add these 5 rows of AlphaSubmitThen hints:

```rust
// AlphaSubmitThen bindings — push buffer AND execute in one key
lines.push(Line::raw(""));  // spacer
lines.push(Line::from(vec![
    Span::styled("+", key_style), Span::raw(" add    "),
    Span::styled("-", key_style), Span::raw(" sub"),
]));
lines.push(Line::from(vec![
    Span::styled("*", key_style), Span::raw(" mul    "),
    Span::styled("/", key_style), Span::raw(" div"),
]));
lines.push(Line::from(vec![
    Span::styled("^", key_style), Span::raw(" pow    "),
    Span::styled("!", key_style), Span::raw(" fact"),
]));
lines.push(Line::from(vec![
    Span::styled("n", key_style), Span::raw(" neg    "),
    Span::styled("s", key_style), Span::raw(" swap"),
]));
lines.push(Line::from(vec![
    Span::styled("d", key_style), Span::raw(" drop   "),
    Span::styled("r", key_style), Span::raw(" rot"),
]));
```

The exact formatting should match the existing style in hints_pane.rs. Read the file before editing to match the exact span/line construction pattern.

#### 8c: Add AlphaStore state render branch

In `hints_pane::render`, add a branch for `AppMode::AlphaStore(_)` before or after the `AppMode::Alpha(_)` branch:

```rust
AppMode::AlphaStore(_) => {
    lines.push(Line::styled("STORE NAME", dim));
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("Enter", key_style), Span::raw(" store"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Esc",   key_style), Span::raw(" cancel"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Bksp",  key_style), Span::raw(" delete"),
    ]));
}
```

---

### Borrow Checker Note for AlphaSubmit

The tricky part is `self.mode` — we need to read its contents AND set it to Normal in the same `apply()` call. The `std::mem::replace` approach is idiomatic:

```rust
let mode = std::mem::replace(&mut self.mode, AppMode::Normal);
```

This atomically takes ownership of the old mode and sets `self.mode = Normal`. No clone needed. This pattern is already likely used elsewhere in the codebase — check app.rs for similar patterns.

---

### Test Patterns

All tests follow existing patterns. Examples:

```rust
// commands.rs test
#[test]
fn test_parse_del_command() {
    assert_eq!(
        parse_command("myvar DEL"),
        Ok(Action::DeleteRegister("myvar".to_string()))
    );
}

// app.rs test — EnterStoreMode with empty stack
#[test]
fn test_enter_store_mode_empty_stack() {
    let mut app = App::new();
    app.apply(Action::EnterStoreMode);
    assert!(matches!(app.mode, AppMode::Normal));
    assert!(app.error_message.is_some());
}

// app.rs test — EnterStoreMode with value on stack
#[test]
fn test_enter_store_mode_with_value() {
    let mut app = App::new();
    app.apply(Action::EnterAlpha);  // or however you push a value...
    // Push 42 via dispatch
    app.dispatch(Action::Push(CalcValue::from(42)));
    app.apply(Action::EnterStoreMode);
    assert!(matches!(app.mode, AppMode::AlphaStore(_)));
}

// app.rs test — full store flow
#[test]
fn test_alpha_store_submit() {
    let mut app = App::new();
    app.dispatch(Action::Push(CalcValue::from(42)));
    app.mode = AppMode::AlphaStore(String::new());
    app.apply(Action::AlphaChar('x'));
    app.apply(Action::AlphaSubmit);
    assert!(matches!(app.mode, AppMode::Normal));
    assert!(app.state.registers.contains_key("x"));
}

// hints_pane.rs test — S appears in normal mode
#[test]
fn test_normal_mode_shows_store_hint() {
    // Use TestBackend + full_content() pattern
    let buf = render_hints(AppMode::Normal, &CalcState::new(), 80, 20);
    let content = full_content(&buf);
    assert!(content.contains('S'));
    assert!(content.contains("store"));
}

// hints_pane.rs test — alpha mode shows AlphaSubmitThen bindings
#[test]
fn test_alpha_mode_shows_submit_then_bindings() {
    let buf = render_hints(AppMode::Alpha(String::new()), &CalcState::new(), 80, 20);
    let content = full_content(&buf);
    assert!(content.contains("add"));
    assert!(content.contains("sub"));
}
```

For `render_hints`, follow the same helper pattern as other hints_pane tests.

---

### What Is Already Working (No Changes Needed)

- `StoreRegister` / `RecallRegister` dispatch in `app.rs::dispatch()` — fully implemented
- STORE/RCL parsing in `commands.rs` — fully implemented
- Register display section in `hints_pane.rs` — framework exists, shows `name  value  name RCL`
- Error on empty-stack STORE — already handled in dispatch()
- Error on missing-register RCL — already handled in dispatch()

This story does NOT touch the engine. All changes are UI layer only.

---

### Epic 3 Retro Action Items Addressed

| Retro Item | This Story |
|------------|------------|
| A1 — Enrich alpha mode hints (AlphaSubmitThen bindings) | Task 8b ✓ |
| A2 — Add STORE syntax hint to normal mode | Task 8a ✓ |
| A3 — Dedicated store shortcut key `S` | Tasks 2–5 ✓ |
| A4 — Better error message for non-numeric input | Task 5e ✓ |

A5 (integration test stubs) is deferred to another Epic 4 story.

---

### Previous Story Learnings (Story 3.6)

- `cargo fmt` reformats code — run after all edits
- Test count was 261 after Story 3.6
- `App::new()` is public and works cleanly for test setup
- `std::mem::replace` preferred over `.clone()` for mode transitions
- Read each file before editing — exact match required by Edit tool

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — clean implementation, no issues.

### Completion Notes List

- Added `AppMode::AlphaStore(String)` variant to mode.rs — parallel to `Alpha(String)`, holds register name being typed
- Added `Action::EnterStoreMode` to action.rs; removed stale `#[allow(dead_code)]` from `DeleteRegister`
- Added `[name, "DEL"]` arm to `parse_command` in commands.rs — enables `myvar DEL` command
- Wired `'S'` key → `Action::EnterStoreMode` in handler.rs Normal arm; added `AppMode::AlphaStore(_)` handler arm
- `EnterStoreMode` in apply(): guards empty stack → error stays Normal; else → `AlphaStore("")`
- `AlphaChar`/`AlphaBackspace`: updated to use `match &mut self.mode { Alpha(buf) | AlphaStore(buf) => ... }` pattern with bool return to avoid borrow conflicts
- `AlphaCancel`: already unconditional (sets Normal regardless) — no code change needed; AC satisfied
- `AlphaSubmit`: refactored via `std::mem::replace` pattern; AlphaStore path PEEKS (not pops) top-of-stack per AC2 "stack is unchanged"; creates undo snapshot
- Improved error message: "Unknown input: X (use 'name STORE' or 'name RCL')" for unrecognized Alpha input
- Added `EnterStoreMode` to dispatch() unreachable arm
- mode_bar.rs: `Alpha(_) | AlphaStore(_) => "[INSERT]"` via `|` pattern
- input_line.rs: `Alpha(buf) | AlphaStore(buf) => format!("> {}_", buf)` via `|` pattern
- hints_pane.rs: moved `dim` definition to top of render(); added AlphaStore branch (STORE NAME header + 3 lines); enriched Alpha branch with 5 rows of AlphaSubmitThen bindings; added `("S", "store")` to STACK_OPS
- 28 new tests; 289 total (261 pre-existing + 28 new); all pass

### File List

- `src/input/commands.rs` — DEL branch + 2 tests
- `src/input/mode.rs` — `AlphaStore(String)` variant
- `src/input/action.rs` — `EnterStoreMode` variant; removed dead_code allow on DeleteRegister
- `src/input/handler.rs` — `'S'` key binding; `AlphaStore(_)` arm + 6 tests
- `src/tui/app.rs` — EnterStoreMode handler; updated AlphaChar/AlphaBackspace/AlphaSubmit; dispatch unreachable + 12 tests
- `src/tui/widgets/mode_bar.rs` — AlphaStore → INSERT + 1 test
- `src/tui/widgets/input_line.rs` — AlphaStore render + 2 tests
- `src/tui/widgets/hints_pane.rs` — AlphaStore branch; enriched Alpha; S in STACK_OPS + 7 tests; added `%  mod` and `p  dup` rows (code review fix)

## Senior Developer Review (AI)

**Reviewer:** claude-sonnet-4-6
**Review Date:** 2026-03-19
**Outcome:** Changes Requested → Fixed

### Action Items

- [x] [Med] `%` (mod) and `p` (dup) were functional AlphaSubmitThen bindings in Alpha mode (handler.rs:58, 63) but absent from hints_pane.rs alpha section, contrary to AC 4. Added `Line::raw("%  mod    p  dup")` row and extended test assertions. [hints_pane.rs:133]

### Notes

- All 8 ACs verified implemented correctly.
- All tasks marked [x] confirmed done in source code.
- Low finding (AlphaBackspace auto-exit to Normal when buffer empties in AlphaStore) left as-is — intentional behavior, consistent with Alpha mode, covered by tests.
- 289 tests pass after fix.
