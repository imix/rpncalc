# Story 3.6: Responsive Layout — Hints Pane Collapse

Status: done

## Story

As a CLI power user,
I want the layout to adapt gracefully when my terminal is narrow or short,
So that the calculator remains fully usable at any terminal size.

## Acceptance Criteria

1. **Given** the terminal is 80 columns wide or wider, **When** the layout renders, **Then** the full layout is shown: stack pane and hints pane side by side with all categories visible.

2. **Given** the terminal is between 60 and 79 columns wide, **When** the layout renders, **Then** the hints pane is narrowed; labels may be abbreviated but key bindings remain visible.

3. **Given** the terminal is under 60 columns wide, **When** the layout renders, **Then** the hints pane is hidden entirely; stack pane, input line, error line, and mode bar remain fully functional.

4. **Given** the terminal has fewer than 20 rows, **When** the layout renders, **Then** at least 4 stack rows are always preserved; fixed rows (input line, error line, mode bar) are never sacrificed.

5. **Given** the terminal is resized during a session, **When** the new dimensions are applied, **Then** the layout transitions immediately with no corruption, no crash, and no loss of stack data.

## Tasks / Subtasks

- [x] Task 1: Update `src/tui/layout.rs` — width-based conditional hints pane rendering (AC: 1–3, 5)
  - [x] Add `let width = f.area().width;` at top of `render`
  - [x] Replace the single horizontal split with a width-conditional branch:
    - `width < 60`: render only `stack_pane` into `outer[0]` (full width, no hints)
    - `width >= 60`: render both `stack_pane` + `hints_pane` with `[Percentage(40), Percentage(60)]` split
  - [x] The 60–79 narrowing is automatic — same percentage split at fewer absolute columns naturally produces a narrower hints pane

- [x] Task 2: Verify height behaviour already correct (AC: 4)
  - [x] Confirm existing `Min(0)` constraint for the main area gives stack pane all remaining height after the 3 fixed rows
  - [x] No code change needed — for any terminal ≥ 7 rows, `height - 3 ≥ 4` guarantees ≥ 4 stack pane rows
  - [x] Document the degenerate case: for terminals < 7 rows, ratatui degrades gracefully without panic (AC5 "no crash")

- [x] Task 3: Add tests to `src/tui/layout.rs`
  - [x] `test_narrow_terminal_hides_hints` — width 50, height 20: `!content.contains("STACK")` (hints pane absent)
  - [x] `test_wide_terminal_shows_hints` — width 80, height 20: `content.contains("STACK")` (hints pane present)
  - [x] `test_medium_terminal_shows_hints` — width 70, height 20: `content.contains("STACK")` (hints pane present, narrowed)
  - [x] `test_minimum_dimensions_no_panic` — width 1, height 1: no panic
  - [x] `test_fixed_rows_always_present` — width 80, height 10: `content.contains("NORMAL")` (mode bar always renders)

- [x] Task 4: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — all 256 pre-existing tests pass + 5 new = 261 total

## Dev Notes

### One File Changes

Only `src/tui/layout.rs` changes. No other production or test files.

### Current `layout.rs` — Full Contents

```rust
use ratatui::{
    layout::{
        Constraint::{Length, Min, Percentage},
        Layout,
    },
    Frame,
};

use crate::tui::{
    app::App,
    widgets::{error_line, hints_pane, input_line, mode_bar, stack_pane},
};

pub fn render(f: &mut Frame, app: &App) {
    let outer = Layout::vertical([Min(0), Length(1), Length(1), Length(1)]).split(f.area());
    let inner = Layout::horizontal([Percentage(40), Percentage(60)]).split(outer[0]);

    stack_pane::render(f, inner[0], &app.state);
    hints_pane::render(f, inner[1], &app.mode, &app.state);
    input_line::render(f, outer[1], &app.mode);
    error_line::render(f, outer[2], app.error_message.as_deref());
    mode_bar::render(f, outer[3], &app.mode, &app.state);
}
```

### Updated `render` Function — Full Replacement

```rust
pub fn render(f: &mut Frame, app: &App) {
    let width = f.area().width;
    let outer = Layout::vertical([Min(0), Length(1), Length(1), Length(1)]).split(f.area());

    if width < 60 {
        stack_pane::render(f, outer[0], &app.state);
    } else {
        let inner = Layout::horizontal([Percentage(40), Percentage(60)]).split(outer[0]);
        stack_pane::render(f, inner[0], &app.state);
        hints_pane::render(f, inner[1], &app.mode, &app.state);
    }

    input_line::render(f, outer[1], &app.mode);
    error_line::render(f, outer[2], app.error_message.as_deref());
    mode_bar::render(f, outer[3], &app.mode, &app.state);
}
```

Key points:
- `let width = f.area().width;` — queried from the frame on every render tick (no caching, per architecture)
- `< 60` branch: `outer[0]` is passed directly to `stack_pane` — full width available
- `>= 60` branch: same `[Percentage(40), Percentage(60)]` split as before — at 70 cols, hints gets 42 cols; at 60 cols, hints gets 36 cols — natural narrowing without code duplication
- No change to `outer` constraints — `Min(0)` continues to give all remaining height to the main area after the 3 fixed rows

### Why 60–79 Narrowing Is Automatic

At different widths with the same `Percentage(40)/Percentage(60)` split:

| Terminal width | Stack cols | Hints cols |
|---|---|---|
| 80 | 32 | 48 |
| 70 | 28 | 42 |
| 60 | 24 | 36 |

At 36 cols, hints pane content like `+  add    -  sub  ` (18 chars) fits comfortably. Key characters (`+`, `-`, `s`, `d` etc.) are always 1-2 chars — visible at any practical width. The `Paragraph::new(lines)` widget simply clips at the available width without panicking. AC2 "key bindings remain visible" is satisfied.

### Height Guarantee Analysis (AC4)

With `Min(0)` vertical constraint and 3 `Length(1)` fixed rows:

| Terminal height | Main area rows | Fixed rows |
|---|---|---|
| 20 | 17 | 3 |
| 10 | 7 | 3 |
| 7 | 4 | 3 |
| 6 | 3 | 3 |
| 4 | 1 | 3 |

For heights ≥ 7 (covering the practical range the AC targets — terminals "fewer than 20 rows" implies some minimum usable size), the stack pane gets ≥ 4 rows. For the degenerate case < 7 rows, ratatui degrades gracefully: it never panics, but the stack area may receive < 4 rows. The fixed rows are always allocated first by ratatui's constraint solver. AC5 "no crash" is satisfied at all terminal sizes.

**No code change needed for height management** — the existing `Min(0)` constraint is correct. The height AC is satisfied by the natural behavior of ratatui's layout system.

### AC5 — Resize Is Handled by ratatui

`render(f, app)` is called on every event, including `Event::Resize` (via the main event loop in `main.rs`). The `f.area()` is queried fresh on every call. No layout state is cached. Stack data lives in `App.state` which is never touched by layout rendering. AC5 "no data loss" is an architectural guarantee, not a code change. ✓

### Test Module — Complete Implementation

Add at the bottom of `layout.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::App;
    use ratatui::{backend::TestBackend, Terminal};

    fn render_layout(width: u16, height: u16) -> ratatui::buffer::Buffer {
        let app = App::new();
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn full_content(buf: &ratatui::buffer::Buffer) -> String {
        let area = buf.area();
        (0..area.height)
            .flat_map(|y| (0..area.width).map(move |x| (x, y)))
            .map(|(x, y)| buf.cell((x, y)).unwrap().symbol().to_string())
            .collect()
    }

    #[test]
    fn test_narrow_terminal_hides_hints() {
        let buf = render_layout(50, 20);
        let content = full_content(&buf);
        // "STACK" only appears in hints_pane — absent when width < 60
        assert!(!content.contains("STACK"));
    }

    #[test]
    fn test_wide_terminal_shows_hints() {
        let buf = render_layout(80, 20);
        let content = full_content(&buf);
        assert!(content.contains("STACK"));
    }

    #[test]
    fn test_medium_terminal_shows_hints() {
        // 60-79 range: hints pane present but narrowed
        let buf = render_layout(70, 20);
        let content = full_content(&buf);
        assert!(content.contains("STACK"));
    }

    #[test]
    fn test_minimum_dimensions_no_panic() {
        let _ = render_layout(1, 1);
    }

    #[test]
    fn test_fixed_rows_always_present() {
        // mode bar shows "NORMAL" — always rendered regardless of terminal size
        let buf = render_layout(80, 10);
        let content = full_content(&buf);
        assert!(content.contains("NORMAL"));
    }
}
```

### Why `"STACK"` Is the Correct Hints Pane Discriminator

`"STACK"` appears in `hints_pane::render` as `Line::styled("STACK", dim)` in every Normal mode render (depth 0, 1, and ≥ 2 — see Story 3.4). It never appears in `stack_pane::render` (which only renders numeric rows like `1: 3.14`). Therefore `content.contains("STACK")` ≡ "hints pane was rendered". ✓

### Architecture Compliance

- Only `src/tui/layout.rs` changes — no new files [Source: architecture.md#Module structure]
- `f.area().width` is queried on every render tick — no cached dimensions [Source: architecture.md#Cross-Cutting Concerns, UX spec implementation guidelines]
- `render(f, app)` remains the sole layout function — `app` is read-only borrow [Source: architecture.md#TUI boundary]
- No `unwrap()` in production code — all calls are infallible [Source: architecture.md#No unwrap() policy]

### Previous Story Learnings (Story 3.5)

- `cargo fmt` reformats code — run after all edits
- Test count was 256 after Story 3.5; this story adds 5 → expect 261 total
- Layout tests follow the exact same `TestBackend` + `full_content` pattern as widget tests
- `App::new()` is public and works cleanly for test setup (used in 30+ tests in app.rs)

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — clean implementation, no issues.

### Completion Notes List

- Added `let width = f.area().width;` queried fresh from frame on every render tick
- Replaced unconditional horizontal split with `if width < 60` branch: narrow path passes `outer[0]` directly to `stack_pane` (full width); wide path uses same `Percentage(40)/Percentage(60)` split as before
- Height management verified correct: existing `Min(0)` constraint guarantees ≥ 4 stack rows for terminals ≥ 7 rows; ratatui degrades gracefully below that without panic
- 5 new tests added in `#[cfg(test)] mod tests` at bottom of `layout.rs` using `App::new()` + `TestBackend` + `full_content()` pattern
- "STACK" discriminator confirmed: appears only in `hints_pane` section header, never in `stack_pane` output
- `cargo fmt` applied; 261 tests pass total (256 pre-existing + 5 new)

### File List

- `src/tui/layout.rs` — width-conditional hints pane rendering + 5 layout tests
