# Story 3.2: Chord System — Leader Keys & Submenus

Status: done

## Story

As a CLI power user,
I want pressing a chord leader to reveal a submenu of related operations in the hints pane,
So that I can discover and execute any grouped operation in two keystrokes with no prior knowledge.

## Acceptance Criteria

1. **Given** normal mode is active, **When** the user presses a chord leader key (`t`, `l`, `f`, `c`, `m`, `x`, `X`), **Then** the app enters `AppMode::Chord(category)` and the hints pane replaces the normal grid with that category's header (e.g. `[TRIG]`) and operations.

2. **Given** a chord is active and the submenu is showing, **When** the user presses a valid second key, **Then** the corresponding operation executes, the app returns to `AppMode::Normal`, and the hints pane shows the normal categorized view.

3. **Given** a chord is active, **When** the user presses `Esc`, **Then** the chord is cancelled, no operation executes, no error is shown, and the app returns to `AppMode::Normal`.

4. **Given** a chord is active, **When** the user presses a key not in that submenu, **Then** the chord is cancelled, the app returns to `AppMode::Normal`, and an appropriate error message is displayed.

5. **Given** the trig chord is used (`t` then `s`), **When** the operation executes, **Then** sin is applied to the top stack value using the active angle mode.

6. **Given** any chord category, **When** the submenu is showing, **Then** every operation in that category is visible with its key and short label.

## Tasks / Subtasks

- [x] Task 1: Add three new `Action` variants to `src/input/action.rs` (AC: 1–4)
  - [x] Add `use crate::input::mode::ChordCategory;` import to `action.rs`
  - [x] Add `EnterChordMode(ChordCategory)` — transitions app into chord-wait state
  - [x] Add `ChordCancel` — Esc in chord mode: return to Normal, no error
  - [x] Add `ChordInvalid` — unknown chord second key: return to Normal, set error

- [x] Task 2: Rewrite `src/input/handler.rs` chord dispatch (AC: 1–5)
  - [x] Split `AppMode::Normal | AppMode::Chord(_) => ...` into two separate arms: `AppMode::Normal => ...` and `AppMode::Chord(category) => ...`
  - [x] In `AppMode::Normal` arm, add 7 chord leader bindings before the `_ => Action::Noop` fallthrough
  - [x] Implement `AppMode::Chord(category)` arm with Esc, Char, and fallthrough
  - [x] Add private `fn dispatch_chord_key(category: &ChordCategory, c: char) -> Action`
  - [x] Add required imports: `AngleMode`, `Base`, `HexStyle`, `ChordCategory`

- [x] Task 3: Update `src/tui/app.rs` (AC: 1–4)
  - [x] Add `Action::EnterChordMode(cat)` arm in `apply()`
  - [x] Add `Action::ChordCancel` arm
  - [x] Add `Action::ChordInvalid` arm
  - [x] Add `was_chord` flag in fallthrough arm for mode reset after dispatch
  - [x] Add the three new variants to the `unreachable!` list in `dispatch()`

- [x] Task 4: Update `src/tui/widgets/hints_pane.rs` — chord submenu render (AC: 1, 6)
  - [x] Add 7 submenu data constants: `TRIG_OPS`, `LOG_OPS`, `FN_OPS`, `CONST_OPS`, `ANGLE_OPS`, `BASE_OPS`, `HEX_STYLE_OPS`
  - [x] Update import: `use crate::input::mode::{AppMode, ChordCategory};`
  - [x] Add `AppMode::Chord(category)` render branch

- [x] Task 5: Unit tests (AC: 1–6)
  - [x] `handler.rs`: all 10 chord tests (leaders, all 8 category second-key sets, Esc cancel, invalid key)
  - [x] `app.rs`: 5 chord mode tests
  - [x] `hints_pane.rs`: 4 chord render tests

- [x] Task 6: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — 231 tests pass (213 pre-existing + 18 new)

## Dev Notes

### Four Files Change

| File | Change |
|---|---|
| `src/input/action.rs` | Add 3 variants + 1 import |
| `src/input/handler.rs` | Split Normal/Chord arms; add leader keys; add `dispatch_chord_key` fn |
| `src/tui/app.rs` | Handle 3 new variants in `apply()`; chord-mode return in fallthrough arm |
| `src/tui/widgets/hints_pane.rs` | Add submenu constants + chord render branch |

Do NOT touch `engine/`, `layout.rs`, `mode.rs`, `mode_bar.rs`, `stack_pane.rs`, or any other file.

### Chord Leader Bindings — Key Collision Analysis

None of `t`, `l`, `f`, `c`, `m`, `x`, `X` currently produce actions in Normal mode — all fall through to `_ => Action::Noop`. Safe to add.

### dispatch_chord_key — Complete Implementation

```rust
fn dispatch_chord_key(category: &ChordCategory, c: char) -> Action {
    match category {
        ChordCategory::Trig => match c {
            's' => Action::Execute(Op::Sin),
            'c' => Action::Execute(Op::Cos),
            'a' => Action::Execute(Op::Tan),
            'S' => Action::Execute(Op::Asin),
            'C' => Action::Execute(Op::Acos),
            'A' => Action::Execute(Op::Atan),
            _ => Action::ChordInvalid,
        },
        ChordCategory::Log => match c {
            'l' => Action::Execute(Op::Ln),
            'L' => Action::Execute(Op::Log10),
            'e' => Action::Execute(Op::Exp),
            'E' => Action::Execute(Op::Exp10),
            _ => Action::ChordInvalid,
        },
        ChordCategory::Functions => match c {
            's' => Action::Execute(Op::Sqrt),
            'q' => Action::Execute(Op::Square),
            'r' => Action::Execute(Op::Reciprocal),
            'a' => Action::Execute(Op::Abs),
            _ => Action::ChordInvalid,
        },
        ChordCategory::Constants => match c {
            'p' => Action::Execute(Op::PushPi),
            'e' => Action::Execute(Op::PushE),
            'g' => Action::Execute(Op::PushPhi),
            _ => Action::ChordInvalid,
        },
        ChordCategory::AngleMode => match c {
            'd' => Action::SetAngleMode(AngleMode::Deg),
            'r' => Action::SetAngleMode(AngleMode::Rad),
            'g' => Action::SetAngleMode(AngleMode::Grad),
            _ => Action::ChordInvalid,
        },
        ChordCategory::Base => match c {
            'c' => Action::SetBase(Base::Dec),
            'h' => Action::SetBase(Base::Hex),
            'o' => Action::SetBase(Base::Oct),
            'b' => Action::SetBase(Base::Bin),
            _ => Action::ChordInvalid,
        },
        ChordCategory::HexStyle => match c {
            'c' => Action::SetHexStyle(HexStyle::ZeroX),
            'a' => Action::SetHexStyle(HexStyle::Dollar),
            's' => Action::SetHexStyle(HexStyle::Hash),
            'i' => Action::SetHexStyle(HexStyle::Suffix),
            _ => Action::ChordInvalid,
        },
    }
}
```

Required additional imports in `handler.rs`:
```rust
use crate::engine::angle::AngleMode;
use crate::engine::base::{Base, HexStyle};
use crate::input::mode::ChordCategory;
```

### app.rs — Chord-Mode Return in Fallthrough Arm

The existing fallthrough arm:
```rust
action => {
    let pre_op = self.state.clone();
    match self.dispatch(action) {
        Ok(()) => {
            self.undo_history.snapshot(&pre_op);
            self.error_message = None;
        }
        Err(e) => {
            self.error_message = Some(e.to_string());
        }
    }
}
```

After update:
```rust
action => {
    let was_chord = matches!(self.mode, AppMode::Chord(_));
    let pre_op = self.state.clone();
    match self.dispatch(action) {
        Ok(()) => {
            self.undo_history.snapshot(&pre_op);
            self.error_message = None;
        }
        Err(e) => {
            self.error_message = Some(e.to_string());
        }
    }
    if was_chord {
        self.mode = AppMode::Normal;
    }
}
```

This ensures that **any** action dispatched via the fallthrough arm while in Chord mode (whether it succeeds or fails) returns the mode to Normal. `SetBase`, `SetAngleMode`, `SetHexStyle`, and `Execute` all flow through this arm.

### hints_pane.rs — Submenu Data Constants

Place these after the existing `CHORD_LEADERS` constant:

```rust
const TRIG_OPS: &[(&str, &str)] = &[
    ("s", "sin"),   ("c", "cos"),
    ("a", "tan"),   ("S", "asin"),
    ("C", "acos"),  ("A", "atan"),
];

const LOG_OPS: &[(&str, &str)] = &[
    ("l", "ln"),    ("L", "log10"),
    ("e", "exp"),   ("E", "exp10"),
];

const FN_OPS: &[(&str, &str)] = &[
    ("s", "sqrt"),  ("q", "sq"),
    ("r", "recip"), ("a", "abs"),
];

const CONST_OPS: &[(&str, &str)] = &[
    ("p", "π"),     ("e", "e"),
    ("g", "φ"),
];

const ANGLE_OPS: &[(&str, &str)] = &[
    ("d", "deg"),   ("r", "rad"),
    ("g", "grad"),
];

const BASE_OPS: &[(&str, &str)] = &[
    ("c", "dec"),   ("h", "hex"),
    ("o", "oct"),   ("b", "bin"),
];

const HEX_STYLE_OPS: &[(&str, &str)] = &[
    ("c", "0xFF"),  ("a", "$FF"),
    ("s", "#FF"),   ("i", "FFh"),
];
```

### hints_pane.rs — Chord Render Branch

Add this after the Alpha branch (`if matches!(mode, AppMode::Alpha(_)) { ... return; }`) and before the Normal mode block:

```rust
if let AppMode::Chord(category) = mode {
    let dim = Style::default().add_modifier(Modifier::DIM);
    let (header, ops): (&str, &[(&str, &str)]) = match category {
        ChordCategory::Trig      => ("[TRIG]",  TRIG_OPS),
        ChordCategory::Log       => ("[LOG]",   LOG_OPS),
        ChordCategory::Functions => ("[FN]",    FN_OPS),
        ChordCategory::Constants => ("[CONST]", CONST_OPS),
        ChordCategory::AngleMode => ("[MODE]",  ANGLE_OPS),
        ChordCategory::Base      => ("[BASE]",  BASE_OPS),
        ChordCategory::HexStyle  => ("[HEX]",   HEX_STYLE_OPS),
    };
    let mut lines: Vec<Line<'static>> = vec![Line::styled(header, dim)];
    lines.extend(entries_to_lines(ops));
    f.render_widget(Paragraph::new(lines), area);
    return;
}
```

Update the import at the top of `hints_pane.rs`:
```rust
// change:
use crate::input::mode::AppMode;
// to:
use crate::input::mode::{AppMode, ChordCategory};
```

### Lifetime Notes for hints_pane.rs

`Line::styled(header, dim)` where `header: &str` — since `header` comes from a `match` on `category` and the string literals are all `&'static str`, the resulting `Line<'static>` is fine. The `Vec<Line<'static>>` type annotation on `lines` is already established in the existing Normal branch code — the Chord branch should use the same type.

### Architecture Compliance

- `handler::handle_key` remains a pure function — `dispatch_chord_key` is a private pure function taking `&ChordCategory` and `char`, returning `Action`. No side effects. [Source: architecture.md#Principle 5]
- No `unwrap()` in non-test code — the match arms are exhaustive. [Source: architecture.md#No unwrap() policy]
- `Action` enum is the sole dispatch mechanism — chord ops route through `Action::Execute`, `Action::SetBase`, `Action::SetAngleMode`, `Action::SetHexStyle`. No new string dispatch. [Source: architecture.md#Communication Patterns]
- `hints_pane::render` remains purely functional — reads `&AppMode` only. [Source: architecture.md#Widgets]

### Previous Story Learnings

- **Story 3.1:** `entries_to_lines` helper is reused for chord submenu ops — no changes needed to it
- **Story 3.1:** `TestBackend` + `full_content(buf)` test pattern already established in `hints_pane.rs` — reuse for chord render tests
- **Story 3.1:** No border on hints pane — chord submenu also renders borderless `Paragraph` directly into `area`
- **Story 2.4 / handler.rs:** The `key()` and `ctrl_key()` test helpers in `handler.rs` tests — reuse for chord second-key tests

### Story 3.3 Preview (Do NOT Implement)

Story 3.3 adds base and angle mode switching — the `SetBase` and `SetAngleMode` actions are already in the engine from Epic 1 and already have dispatch arms in `app.rs::dispatch()`. Story 3.2 wires up the key bindings that produce them. Story 3.3 will verify the full end-to-end flow (stack redisplay in new base, mode bar update). The chord infrastructure built here is the foundation.

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — clean implementation, no issues.

### Completion Notes List

- Removed stale `#[allow(dead_code)]` from `SetBase`/`SetAngleMode`/`SetHexStyle` in action.rs — they're now live
- Split `AppMode::Normal | AppMode::Chord(_)` into two separate match arms; handler now routes chord second-keys through `dispatch_chord_key`
- `was_chord` flag in app.rs fallthrough arm ensures mode returns to Normal on any chord action (success or failure)
- 7 submenu constants + chord render branch in hints_pane.rs — all show DIM header + ops grid, no border
- 231 tests pass: 10 new handler tests, 5 new app tests, 4 new hints_pane tests

### File List

- `src/input/action.rs` — added `EnterChordMode(ChordCategory)`, `ChordCancel`, `ChordInvalid`; removed dead_code allows from SetBase/SetAngleMode/SetHexStyle
- `src/input/handler.rs` — split Normal/Chord arms; added chord leader bindings; added `dispatch_chord_key` fn
- `src/tui/app.rs` — handled 3 new action variants; added `was_chord` flag in fallthrough arm
- `src/tui/widgets/hints_pane.rs` — added 7 submenu constants + chord render branch + 4 tests
