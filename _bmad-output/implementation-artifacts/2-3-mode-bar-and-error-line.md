# Story 2.3: Mode Bar & Error Line

Status: done

## Story

As a CLI power user,
I want a permanent status strip showing my current mode and clear error feedback when something goes wrong,
So that I always know what state the calculator is in and errors never leave me confused.

## Acceptance Criteria

1. **Given** the TUI is running in normal mode, **When** the mode bar renders, **Then** it shows `[NORMAL]` on the left and the active angle mode and base on the right.

2. **Given** the TUI is in alpha (insert) mode, **When** the mode bar renders, **Then** it shows `[INSERT]` on the left.

3. **Given** HEX base is active, **When** the mode bar renders, **Then** the active hex representation style is also shown (e.g. `0xFF`, `$FF`, `#FF`, or `FFh`).

4. **Given** no error has occurred, **When** the error line renders, **Then** it is completely blank — no text, no separator, no visual noise.

5. **Given** an invalid operation is attempted (e.g. divide by zero, stack underflow), **When** the error line renders, **Then** it displays a clear description of the error in a distinct color (`Color::Red`).

6. **Given** an error is showing, **When** the next valid action completes successfully, **Then** the error line clears automatically — no keystroke required to dismiss it. *(This AC is implemented by `app.rs` setting `error_message = None` on success — the widget itself just renders whatever `msg: Option<&str>` it receives.)*

## Tasks / Subtasks

- [x] Task 1: Implement `mode_bar::render` in `src/tui/widgets/mode_bar.rs` (AC: 1–3)
  - [x] Remove the `Block::bordered().title("Mode")` stub; the mode bar is a 1-row area with no border
  - [x] Compute `mode_str`: `"[NORMAL]"` for `AppMode::Normal` and `AppMode::Chord(_)`; `"[INSERT]"` for `AppMode::Alpha(_)`
  - [x] Compute `right_str`: always include `state.angle_mode` and `state.base` separated by two spaces; if `state.base == Base::Hex` also append two spaces and the hex style example string (see Dev Notes)
  - [x] Compute left-right padding: `pad_len = area.width.saturating_sub((mode_str.len() + right_str.len()) as u16)` — fill with spaces
  - [x] Assemble a single `Line::from(vec![mode_span, pad_span, right_span])` styled with `Color::Yellow` throughout; render as `Paragraph::new(line)` into `area`
  - [x] Unit tests (see Task 2)

- [x] Task 2: Unit tests in `src/tui/widgets/mode_bar.rs` (AC: 1–3)
  - [x] `test_normal_mode_shows_normal` — verify `[NORMAL]` appears in the rendered row
  - [x] `test_alpha_mode_shows_insert` — verify `[INSERT]` appears
  - [x] `test_chord_mode_shows_normal` — verify chord state still shows `[NORMAL]`
  - [x] `test_angle_deg_appears` — verify `DEG` in the row
  - [x] `test_base_dec_appears` — verify `DEC` in the row
  - [x] `test_hex_base_shows_style` — set `state.base = Base::Hex`, verify `HEX` and the style example (e.g. `0xFF`) both appear
  - [x] `test_non_hex_no_style_suffix` — set `state.base = Base::Dec`, verify no hex style string (`0xFF`/`$FF`/`#FF`/`FFh`) appears
  - [x] `test_mode_bar_is_yellow` — verify the mode_str cell has `Color::Yellow`

- [x] Task 3: Implement `error_line::render` in `src/tui/widgets/error_line.rs` (AC: 4–5)
  - [x] Remove the `Block::bordered().title("Error")` stub; error line is a 1-row area with no border
  - [x] When `msg` is `None`: render `Paragraph::new("")` (completely blank row)
  - [x] When `msg` is `Some(text)`: render `Paragraph::new(text)` styled with `Style::default().fg(Color::Red)`
  - [x] Unit tests (see Task 4)

- [x] Task 4: Unit tests in `src/tui/widgets/error_line.rs` (AC: 4–5)
  - [x] `test_no_error_is_blank` — render with `None`; verify row contains only spaces, no visible characters
  - [x] `test_error_message_appears` — render with `Some("Stack underflow")`; verify text appears in row
  - [x] `test_error_message_is_red` — render with `Some("x")`; verify cell has `Color::Red`

- [x] Task 5: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — 169 tests pass (157 existing + 12 new: 9 mode_bar + 3 error_line)

## Dev Notes

### Only Two Files Change

This story touches **exactly two files**: `src/tui/widgets/mode_bar.rs` and `src/tui/widgets/error_line.rs`.

Do **not** touch `layout.rs`, `app.rs`, `main.rs`, engine files, `stack_pane.rs`, or any other widget. The render signatures from Story 2.1 are already wired in `layout.rs`:

```rust
error_line::render(f, outer[2], app.error_message.as_deref());
mode_bar::render(f, outer[3], &app.mode, &app.state);
```

These signatures must remain unchanged.

### No Borders — These Are 1-Row Strips

Both widgets occupy a single row (height=1). `Block::bordered()` consumes 2 rows for top/bottom borders — NEVER use it here. Render directly with `Paragraph::new(...)` into the raw `area`.

The layout in `layout.rs` already allocates them as `Length(1)`:
```rust
let outer = Layout::vertical([Min(0), Length(1), Length(1), Length(1)]).split(f.area());
// outer[1] = input_line, outer[2] = error_line, outer[3] = mode_bar
```

### Mode Bar: Left-Right Layout with Computed Padding

The mode bar shows mode indicator on the left, status info on the right. Achieve this with a single `Line` containing three spans: left content, dynamic padding, right content.

```rust
let mode_str = match mode {
    AppMode::Normal | AppMode::Chord(_) => "[NORMAL]",
    AppMode::Alpha(_) => "[INSERT]",
};

let right_str = if state.base == Base::Hex {
    let hex_example = match state.hex_style {
        HexStyle::ZeroX => "0xFF",
        HexStyle::Dollar => "$FF",
        HexStyle::Hash  => "#FF",
        HexStyle::Suffix => "FFh",
    };
    format!("{}  {}  {}", state.angle_mode, state.base, hex_example)
} else {
    format!("{}  {}", state.angle_mode, state.base)
};

let total_content = mode_str.len() + right_str.len();
let pad_len = (area.width as usize).saturating_sub(total_content);
let padding = " ".repeat(pad_len);

let style = Style::default().fg(Color::Yellow);
let line = Line::from(vec![
    Span::styled(mode_str, style),
    Span::raw(padding),
    Span::styled(right_str, style),
]);
f.render_widget(Paragraph::new(line), area);
```

### Mode Bar Hex Style Display Strings

The UX spec defines these exact display examples (with `FF` as placeholder):

| `HexStyle` variant | Mode bar shows |
|---|---|
| `HexStyle::ZeroX` | `0xFF` |
| `HexStyle::Dollar` | `$FF` |
| `HexStyle::Hash` | `#FF` |
| `HexStyle::Suffix` | `FFh` |

Note: `HexStyle`'s `Display` impl gives only the prefix/suffix marker (`"0x"`, `"$"`, `"#"`, `"h"`). Do **not** use `state.hex_style.to_string()` for the mode bar — use the explicit match above.

### Error Line: Simple Conditional Render

```rust
pub fn render(f: &mut Frame, area: Rect, msg: Option<&str>) {
    let paragraph = match msg {
        None => Paragraph::new(""),
        Some(text) => Paragraph::new(text).style(Style::default().fg(Color::Red)),
    };
    f.render_widget(paragraph, area);
}
```

**AC 6 is owned by `app.rs`:** The `App::apply()` method already sets `self.error_message = None` on every successful operation. The error_line widget is stateless — it simply renders whatever `msg` it receives. No special logic needed in the widget.

### Required Imports

**`mode_bar.rs`:**
```rust
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::engine::{base::{Base, HexStyle}, stack::CalcState};
use crate::input::mode::AppMode;
```

**`error_line.rs`:**
```rust
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};
```

### Test Helper Pattern (from Story 2.2 — reuse this)

Use `TestBackend` + `Terminal::draw` to capture rendered output into a `Buffer`. Use the non-deprecated `buf.cell((x, y)).unwrap()` API (NOT the deprecated `buf.get(x, y)`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};
    // mode_bar tests also need:
    use crate::engine::{angle::AngleMode, base::{Base, HexStyle}, stack::CalcState};
    use crate::input::mode::{AppMode, ChordCategory};

    fn render_mode_bar(mode: &AppMode, state: &CalcState, width: u16) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(width, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, f.area(), mode, state)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn row_content(buf: &ratatui::buffer::Buffer, row: u16) -> String {
        let width = buf.area().width;
        (0..width)
            .map(|x| buf.cell((x, row)).unwrap().symbol().to_string())
            .collect()
    }
```

For `error_line` tests, create an equivalent `render_error_line(msg, width)` helper.

### AppMode Variants — All Three Must Be Handled

```rust
pub enum AppMode {
    Normal,
    Alpha(String),   // string is the buffer content — may be empty
    Chord(ChordCategory), // transient; treat as Normal for mode bar display
}
```

`AppMode::Chord(_)` → show `"[NORMAL]"`. Chord is a transient sub-state of normal mode; the hints pane (not mode bar) communicates the active chord category.

### AngleMode and Base Display

Both already implement `fmt::Display`:
- `AngleMode::Deg` → `"DEG"`, `AngleMode::Rad` → `"RAD"`, `AngleMode::Grad` → `"GRD"`
- `Base::Dec` → `"DEC"`, `Base::Hex` → `"HEX"`, `Base::Oct` → `"OCT"`, `Base::Bin` → `"BIN"`

Use `format!("{}", state.angle_mode)` and `format!("{}", state.base)` to get these strings. Or call `state.angle_mode.to_string()` and `state.base.to_string()`.

### Test Dimensions

- Mode bar height = 1 (no border). `TestBackend::new(width, 1)` is correct.
- Minimum useful width for a readable mode bar: 30 characters. Tests can use 40.
- The row to inspect is always row 0 (the single row).

```rust
let content = row_content(&buf, 0);
assert!(content.contains("[NORMAL]"), ...);
```

For color checks, find the first non-space cell in the row:
```rust
let cell = buf.cell((0u16, 0u16)).unwrap();
assert_eq!(cell.fg, Color::Yellow, "mode bar should be yellow");
```

### Previous Story Learnings (from 2.2)

- **Deprecated API:** Use `buf.cell((x, y)).unwrap()` (returns `Option<&Cell>`), NOT `buf.get(x, y)` (deprecated in ratatui v0.29). The tuple `(u16, u16)` implements `Into<Position>`.
- **No `Block::bordered()` for full-area widgets** — the block consumes border rows. Widgets that own their area use `Paragraph` directly.
- **`block.inner(area)` pattern** is for widgets that want a border + inner content. Mode bar and error line have no border.
- **Test backend height must match widget height** — for 1-row widgets, use `TestBackend::new(width, 1)` not `TestBackend::new(width, 10)`. The inspected row is 0.
- **FBig gotcha does NOT apply here** — mode bar and error line display no `CalcValue`. No value display path in this story.

### Architecture Compliance

- Widget render functions: `pub fn render(f: &mut Frame, area: Rect, ...)` — pure render, no mutation, no global state reads
- `CalcState` passed as `&CalcState` (read-only) to mode_bar; `error_line` receives only a `&str` slice
- Colors via `ratatui::style::Color` enum (semantic terminal colors, no hardcoded RGB)
- Layout is constraint-based via ratatui — this story does not add any layout logic; it only fills in the widget implementations

### Project Structure

```
src/tui/widgets/
├── mod.rs            ← DO NOT TOUCH
├── stack_pane.rs     ← DO NOT TOUCH (Story 2.2, complete)
├── hints_pane.rs     ← DO NOT TOUCH (stub for future story)
├── input_line.rs     ← DO NOT TOUCH (stub for future story)
├── mode_bar.rs       ← ✅ THIS STORY (replace stub)
└── error_line.rs     ← ✅ THIS STORY (replace stub)
```

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

(none — clean implementation, no issues)

### Completion Notes List

- Replaced `Block::bordered()` stubs in both widgets with proper 1-row `Paragraph`-based renders
- Mode bar: left-right layout via computed padding span; `Color::Yellow` throughout; all 3 `AppMode` variants handled (`Normal`/`Chord(_)` → `[NORMAL]`, `Alpha(_)` → `[INSERT]`); HexStyle example strings match UX spec
- Error line: stateless conditional render; `Color::Red` on error; blank (empty string) when no error
- 9 mode_bar tests + 3 error_line tests (12 new) — all pass; zero regressions
- Added `test_hex_style_variants` (bonus beyond story spec) to cover all 4 HexStyle variants in one parameterized test
- `cargo build`, `cargo clippy -- -D warnings`, `cargo fmt`, `cargo test` all exit 0

## File List

- `src/tui/widgets/mode_bar.rs` — full implementation replacing stub
- `src/tui/widgets/error_line.rs` — full implementation replacing stub
- `_bmad-output/implementation-artifacts/2-3-mode-bar-and-error-line.md` — story file (status, tasks, record)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — updated to `review`

## Change Log

- 2026-03-19: Implemented mode bar and error line — Yellow left/right mode strip with hex style display, Red error row; 12 unit tests added
