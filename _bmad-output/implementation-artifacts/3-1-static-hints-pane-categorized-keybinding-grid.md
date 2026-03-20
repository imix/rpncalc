# Story 3.1: Static Hints Pane — Categorized Keybinding Grid

Status: done

## Story

As a CLI power user,
I want to see a categorized display of available operations in the hints pane,
So that I can find any operation at a glance without consulting documentation.

## Acceptance Criteria

1. **Given** the TUI is running, **When** the hints pane renders, **Then** it shows operations grouped by category: `ARITHMETIC`, `STACK`, and chord leader entries for `TRIG`, `LOG`, `FN`, `CONST`, `MODE`, `BASE`, `HEX`.

2. **Given** the hints pane renders, **When** a chord leader is listed, **Then** it is visually indicated with `›` after the key (e.g., `t›  trig`) so the user knows to press it first.

3. **Given** the hints pane renders, **When** each direct operation is shown, **Then** the key and a short label are both visible (e.g., `+  add`, `s  swap`).

4. **Given** the TUI is in alpha mode, **When** the hints pane renders, **Then** the category grid is replaced by alpha mode hints: `Enter  push`, `Esc  cancel`, `Bksp  delete`.

5. **Given** the pane is narrower than the minimum useful width, **When** the hints pane renders, **Then** it degrades gracefully — no panic, no garbled output; content may be truncated or omitted but the widget does not crash.

## Tasks / Subtasks

- [x] Task 1: Update `hints_pane::render` signature to accept `&CalcState` in addition to `&AppMode` (AC: 1–5)
  - [x] Change signature to: `pub fn render(f: &mut Frame, area: Rect, mode: &AppMode, _state: &CalcState)`
  - [x] Update `src/tui/layout.rs` call site to: `hints_pane::render(f, hints_area, &app.mode, &app.state)`
  - [x] Add `use crate::engine::stack::CalcState;` import to `hints_pane.rs`
  - [x] The `_state` prefix silences the unused warning; future stories (3.4, 3.5) will use it

- [x] Task 2: Implement static hints grid for Normal mode (AC: 1–3)
  - [x] Define two inline data structures: `DIRECT_OPS` (key, label pairs rendered as a two-column grid) and `CHORD_LEADERS` (key, label pairs rendered with `›` indicator)
  - [x] `DIRECT_OPS` — ARITHMETIC section: `+` add, `-` sub, `*` mul, `/` div, `^` pow, `%` mod, `!` fact, `n` neg; STACK section: `s` swap, `d` drop, `p` dup, `r` rot, `u` undo, `y` yank
  - [x] `CHORD_LEADERS` — `t›` trig, `l›` log, `f›` fn, `c›` const, `m›` mode, `x›` base, `X›` hex
  - [x] Render `ARITHMETIC` header line (bold), then pairs two-per-row; blank separator; render `STACK` header (bold), then pairs two-per-row; blank separator; render chord leaders two-per-row with no section header
  - [x] Each direct-op cell format: `{key:<2} {label:<6}` (key left-padded to 2, label left-padded to 6) giving ~10 chars per cell; two cells fit in any pane ≥ 22 chars wide
  - [x] Each chord-leader cell format: `{key}›  {label:<5}` giving ~10 chars per cell
  - [x] Assemble as `Vec<Line>` and render as `Paragraph::new(lines)` — **no border** on hints pane (UX spec: borderless or minimal)
  - [x] Section headers styled `Style::default().add_modifier(Modifier::DIM)` — subdued, not competing with values
  - [x] All text styled with default color (no `Color::Cyan` or special colors in this story)

- [x] Task 3: Implement alpha mode hints (AC: 4)
  - [x] When `mode == AppMode::Alpha(_)`, render a short fixed list instead of the category grid:
    - Line 1: `Enter  push`
    - Line 2: `Esc    cancel`
    - Line 3: `Bksp   delete`
  - [x] These are the only three hints needed in alpha mode
  - [x] Same `Paragraph::new(lines)` approach, no border

- [x] Task 4: Unit tests in `src/tui/widgets/hints_pane.rs` (AC: 1–5)
  - [x] Add test helpers: `fn render_hints(mode, width, height) -> Buffer` using `TestBackend` (same pattern as `stack_pane.rs` tests); `fn full_content(buf) -> String` concatenating all rows
  - [x] `test_normal_mode_shows_arithmetic_header` — render in Normal mode; verify buffer contains `"ARITHMETIC"`
  - [x] `test_normal_mode_shows_stack_header` — render in Normal mode; verify buffer contains `"STACK"`
  - [x] `test_normal_mode_shows_chord_leaders` — render in Normal mode; verify buffer contains `"›"` (the chord indicator character)
  - [x] `test_normal_mode_shows_add_op` — verify `"+"` and `"add"` both appear
  - [x] `test_alpha_mode_shows_push_hint` — render with `AppMode::Alpha("".into())`; verify `"push"` appears and `"ARITHMETIC"` does NOT appear
  - [x] `test_alpha_mode_shows_cancel_hint` — verify `"cancel"` appears in alpha mode
  - [x] `test_narrow_pane_no_panic` — render in a 5×3 buffer; verify no panic (AC: 5)

- [x] Task 5: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — 213 tests pass (206 pre-existing + 7 new hints_pane tests)

## Dev Notes

### Only Two Files Change

| File | Change |
|---|---|
| `src/tui/widgets/hints_pane.rs` | Full implementation replacing the stub |
| `src/tui/layout.rs` | One-line call-site update to pass `&app.state` |

Do NOT touch `app.rs`, `handler.rs`, `action.rs`, engine files, or any other widget.

### Current Stub (to Replace)

```rust
// TODO: Story 3.1 — HintsPane widget full implementation
use ratatui::{layout::Rect, widgets::Block, Frame};
use crate::input::mode::AppMode;

pub fn render(f: &mut Frame, area: Rect, _mode: &AppMode) {
    f.render_widget(Block::bordered().title("Hints"), area);
}
```

The stub renders a `Block::bordered()`. The replacement renders **no border** (the UX spec says "hints pane borderless or minimal"). The content fills the raw area directly.

### layout.rs Call-Site Update

Current (`src/tui/layout.rs`):
```rust
hints_pane::render(f, hints_area, &app.mode);
```

After update:
```rust
hints_pane::render(f, hints_area, &app.mode, &app.state);
```

`layout.rs` already imports `use crate::tui::app::App;` — `App` owns `state: CalcState`, so no additional imports needed in `layout.rs`. In `hints_pane.rs`, add `use crate::engine::stack::CalcState;`.

### No Border — Why

Architecture decision (UX spec): "hints pane borderless or minimal". The stack pane gets `Block::bordered()` because it needs clear visual separation from hints. The hints pane flows directly into its allocated area — the layout split provides the visual boundary.

Remove the `Block::bordered()` stub entirely. Render `Paragraph::new(lines)` directly into `area`.

### Data Layout — Static Tables

Define these as module-level constants or a `fn` returning `&'static [...]`. Example approach:

```rust
// (key_display, label)
const ARITHMETIC: &[(&str, &str)] = &[
    ("+", "add"), ("-", "sub"),
    ("*", "mul"), ("/", "div"),
    ("^", "pow"), ("%", "mod"),
    ("!", "fact"), ("n", "neg"),
];

const STACK_OPS: &[(&str, &str)] = &[
    ("s", "swap"), ("d", "drop"),
    ("p", "dup"),  ("r", "rot"),
    ("u", "undo"), ("y", "yank"),
];

// (key, label) — will be rendered as "t›  trig" etc.
const CHORD_LEADERS: &[(&str, &str)] = &[
    ("t", "trig"), ("l", "log"),
    ("f", "fn"),   ("c", "const"),
    ("m", "mode"), ("x", "base"),
    ("X", "hex"),
];
```

### Two-Column Row Assembly

Given a slice `entries: &[(&str, &str)]`, render two per row:

```rust
fn entries_to_lines(entries: &[(&str, &str)]) -> Vec<Line<'static>> {
    entries
        .chunks(2)
        .map(|chunk| {
            let left = format!("{:<2} {:<6}", chunk[0].0, chunk[0].1);
            let right = chunk.get(1)
                .map(|(k, l)| format!("  {:<2} {:<6}", k, l))
                .unwrap_or_default();
            Line::raw(format!("{}{}", left, right))
        })
        .collect()
}
```

Each `left` cell is 9 chars (`{:<2}` key + space + `{:<6}` label), `right` adds 2 spaces + 9 = 11. Total per row: ~20 chars — comfortably fits in a 28-col inner area (60% of an 80-col terminal = 48 cols).

For chord leaders, the `›` (U+203A) goes between key and spaces:

```rust
fn chord_leaders_to_lines(leaders: &[(&str, &str)]) -> Vec<Line<'static>> {
    leaders
        .chunks(2)
        .map(|chunk| {
            let left = format!("{}›  {:<5}", chunk[0].0, chunk[0].1);
            let right = chunk.get(1)
                .map(|(k, l)| format!("  {}›  {:<5}", k, l))
                .unwrap_or_default();
            Line::raw(format!("{}{}", left, right))
        })
        .collect()
}
```

### Section Header Style

```rust
Line::styled("ARITHMETIC", Style::default().add_modifier(Modifier::DIM))
```

DIM makes headers visually subordinate to the values in the stack pane and the key labels themselves. Do not use BOLD for headers.

### Alpha Mode Hints

When `AppMode::Alpha(_)`:

```rust
let lines = vec![
    Line::raw("Enter  push"),
    Line::raw("Esc    cancel"),
    Line::raw("Bksp   delete"),
];
f.render_widget(Paragraph::new(lines), area);
return;
```

Keep it minimal. Alpha mode is transient — the user just needs the three exit keys.

### Required ratatui Imports

```rust
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::Paragraph,
    Frame,
};
use crate::engine::stack::CalcState;
use crate::input::mode::AppMode;
```

Note: no `Block` import needed (we removed the border). No `Color` needed (no colored hints in this story).

### Test Pattern (Same as stack_pane.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::stack::CalcState;
    use ratatui::{backend::TestBackend, Terminal};

    fn render_hints(mode: AppMode, width: u16, height: u16) -> ratatui::buffer::Buffer {
        let state = CalcState::new();
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, f.area(), &mode, &state)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn full_content(buf: &ratatui::buffer::Buffer) -> String {
        let area = buf.area();
        (0..area.height)
            .flat_map(|y| (0..area.width).map(move |x| (x, y)))
            .map(|(x, y)| buf.cell((x, y)).unwrap().symbol().to_string())
            .collect()
    }
}
```

### Architecture Compliance

- `hints_pane::render` is purely functional — no owned state, no side effects [Source: architecture.md#Principle 5]
- Widget receives `&AppMode` and `&CalcState` as read-only references [Source: architecture.md#Widgets]
- No `unwrap()` in non-test code — `TestBackend::new(...).unwrap()` in tests is fine
- No `Color::Cyan` or accent colors on hints text — those are reserved for the X register in stack pane [Source: ux-design-specification.md#Color Roles]

### Chord Leader `›` Character

Use U+203A `›` (SINGLE RIGHT-POINTING ANGLE QUOTATION MARK), not `>` or `»`. This matches the UX spec's visual language for chord leaders. It's a single char — no width issues in monospace.

### Story 3.2 Preview (Do NOT Implement)

Story 3.2 will add chord detection: when `AppMode::Chord(ChordCategory::Trig)`, the hints pane renders the trig submenu instead of the normal grid. The signature set up here (`&AppMode`) already supports this. Do not implement chord submenu rendering in this story.

### Previous Story Learnings

- **Story 2.2 (stack_pane.rs):** `TestBackend` pattern works perfectly for widget tests — reuse it here
- **Story 2.3 (mode_bar.rs):** Borderless widgets (no `Block::bordered()`) render a `Paragraph` directly into `area` — that's the pattern to follow here
- **Epic 2 bug fixes:** `hints_pane.rs` signature will change in this story (adding `&CalcState`); layout.rs has one call site to update — double-check it compiles before running tests

### Project Structure

```
src/
└── tui/
    ├── layout.rs       ← update one call site
    └── widgets/
        └── hints_pane.rs  ← only file to implement
```

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — clean implementation, no issues.

### Completion Notes List

- Replaced stub with full implementation; removed `Block::bordered()` per UX spec (borderless hints pane)
- Static tables `ARITHMETIC`, `STACK_OPS`, `CHORD_LEADERS` as module-level constants
- `entries_to_lines` and `chord_leaders_to_lines` helpers format two entries per row
- Section headers styled `Modifier::DIM` (not bold) per spec
- Alpha mode returns early with three-line hint list
- 7 unit tests all pass; narrow-pane (5×3) test confirms no panic

### File List

- `src/tui/widgets/hints_pane.rs` — full implementation replacing stub
- `src/tui/layout.rs` — call site updated to pass `&app.state`
- `_bmad-output/implementation-artifacts/3-1-static-hints-pane-categorized-keybinding-grid.md` — story file updated
