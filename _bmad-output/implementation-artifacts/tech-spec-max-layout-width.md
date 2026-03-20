---
title: 'Configurable Maximum Layout Width'
slug: 'max-layout-width'
created: '2026-03-20'
status: 'Completed'
stepsCompleted: [1, 2, 3, 4]
tech_stack: ['Rust', 'ratatui 0.29']
files_to_modify: ['src/tui/layout.rs']
code_patterns: ['Layout::horizontal centering', 'MAX_WIDTH constant', 'Constraint::Length/Min']
test_patterns: ['TestBackend render_layout helper', 'full_content string scan', 'buf.cell() symbol inspection']
---

# Tech-Spec: Configurable Maximum Layout Width

**Created:** 2026-03-20

## Overview

### Problem Statement

On terminals wider than ~100 columns (common on 14"+ screens with small fonts or large displays), the rpncalc TUI stretches to fill the full terminal width. This makes the UI feel dispersed — the stack pane is wide for numbers that are 3 digits, and the hints pane develops large empty right margins. The calculator feels like it was designed for a widescreen dashboard rather than a focused tool.

### Solution

Cap the rendering area at `MAX_WIDTH = 100` columns and center it horizontally in the terminal using ratatui's `Layout::horizontal` with `[Min(0), Length(MAX_WIDTH), Min(0)]` constraints. Terminals narrower than `MAX_WIDTH` are unaffected — the full area is used as before. The constant is defined at the top of `layout.rs` for easy tuning.

### Scope

**In Scope:**
- `src/tui/layout.rs` — `MAX_WIDTH` constant, centering guard, 2 new tests

**Out of Scope:**
- Making `max_width` a config-file option
- Changing the 40/60 stack/hints split ratio
- Changing the `<60` narrow-mode threshold that hides the hints pane

## Context for Development

### Codebase Patterns

- ratatui layout uses constraint-based splitting: `Layout::horizontal([...]).split(area)` returns a `Vec<Rect>`; index with `[n]` to get a specific column
- The `content_area: Rect` replaces `f.area()` as the root area for all downstream layout splits — both the vertical outer split and the horizontal inner split use `content_area`
- The narrow-mode check (`if width < 60`) uses `content_area.width` after centering, so both the narrow and wide code paths are covered correctly by the guard
- Tests use `TestBackend::new(width, height)` + `terminal.draw(|f| render(f, &app))` — `render_layout(w, h)` helper in the existing test module
- Cell inspection: `buf.cell((x, y)).unwrap().symbol()` returns `" "` for blank cells; used to verify margin columns contain only spaces

### Files to Reference

| File | Purpose |
| ---- | ------- |
| `src/tui/layout.rs` | Only file changed — render function, `MAX_WIDTH` constant, and all tests |

### Technical Decisions

- **`MAX_WIDTH: u16 = 100`** — chosen to give a comfortable calculator width (40-col stack, 60-col hints) without sprawling. Matches a common "focused terminal app" convention. Single constant to change if the value needs tuning.
- **`if area.width > MAX_WIDTH` guard** — required correctness guard. Passing `Length(100)` to a layout that's only 50 cols wide causes ratatui to silently overflow or produce zero-width columns. The guard ensures centering only activates when the terminal is actually wider than the cap.
- **`split(area)[1]`** — the idiomatic ratatui way to extract the center column from a three-column `[Min(0), Length(N), Min(0)]` split. Index 0 = left margin, 1 = content, 2 = right margin.

## Implementation Plan

### Tasks

- [x] Task 1: Add `MAX_WIDTH` constant to `src/tui/layout.rs`
  - File: `src/tui/layout.rs`
  - Action: Add `/// Maximum content width — prevents the layout from sprawling on wide terminals.\nconst MAX_WIDTH: u16 = 100;` before `pub fn render`
  - Notes: `u16` matches ratatui's coordinate type — no casting needed

- [x] Task 2: Add centering logic to `render()` in `src/tui/layout.rs`
  - File: `src/tui/layout.rs`
  - Action: Replace the opening lines of `render`:
    ```rust
    // Before:
    let width = f.area().width;
    let outer = Layout::vertical([Min(0), Length(1), Length(1), Length(1)]).split(f.area());

    // After:
    let area = f.area();
    let content_area = if area.width > MAX_WIDTH {
        Layout::horizontal([Min(0), Length(MAX_WIDTH), Min(0)]).split(area)[1]
    } else {
        area
    };
    let width = content_area.width;
    let outer = Layout::vertical([Min(0), Length(1), Length(1), Length(1)]).split(content_area);
    ```
  - Notes: All downstream widget calls (`outer[0]`, `outer[1]`, etc.) are unchanged — they already use `outer` which is now derived from `content_area`

- [x] Task 3: Add two tests to `src/tui/layout.rs`
  - File: `src/tui/layout.rs`
  - Action: Add inside the existing `mod tests` block:
    ```rust
    #[test]
    fn test_wide_terminal_still_shows_hints() {
        // 200-column terminal: content capped at MAX_WIDTH but hints still rendered
        let buf = render_layout(200, 20);
        let content = full_content(&buf);
        assert!(content.contains("STACK"));
        assert!(content.contains("NORMAL"));
    }

    #[test]
    fn test_wide_terminal_has_margins() {
        // 200-column terminal: margins appear as spaces on either side of content.
        // Content (100 cols) is centred in 200 cols → margins are 50 cols each.
        let buf = render_layout(200, 20);
        let left_margin_cell = buf.cell((10u16, 10u16)).unwrap().symbol().to_string();
        assert_eq!(left_margin_cell, " ", "left margin should be blank space");
        let right_margin_cell = buf.cell((190u16, 10u16)).unwrap().symbol().to_string();
        assert_eq!(right_margin_cell, " ", "right margin should be blank space");
    }
    ```
  - Notes: `render_layout` and `full_content` helpers already exist in the test module — no new helpers needed

- [x] Task 4: Quality gates
  - `cargo build` exits 0
  - `cargo clippy -- -D warnings` exits 0
  - `cargo fmt` applied
  - `cargo test` exits 0 — all pre-existing tests pass, 2 new tests added

### Acceptance Criteria

- [x] AC 1: **Given** a terminal wider than 100 columns, **When** the TUI renders, **Then** the content occupies exactly 100 columns centred in the terminal with equal blank margins on each side.

- [x] AC 2: **Given** a terminal exactly 100 columns wide, **When** the TUI renders, **Then** the layout fills the full terminal width with no margins (identical to pre-change behaviour).

- [x] AC 3: **Given** a terminal narrower than 100 columns, **When** the TUI renders, **Then** behaviour is identical to before — full-width content, no margins, no visual change.

- [x] AC 4: **Given** a terminal narrower than 60 columns, **When** the TUI renders, **Then** the hints pane is still hidden (narrow-mode threshold unchanged).

- [x] AC 5: **Given** a wide terminal (>100 cols), **When** the TUI renders, **Then** the hints pane and mode bar are fully visible — no content is clipped or lost.

- [x] AC 6: **Given** `MAX_WIDTH` is changed to any value ≥ 60, **When** a terminal wider than `MAX_WIDTH` renders, **Then** the centering behaviour activates at the new threshold correctly.

## Additional Context

### Dependencies

None — `Constraint::Min` and `Constraint::Length` are already imported in `layout.rs`.

### Testing Strategy

- **Unit tests** (in `layout.rs`): 2 new tests using the existing `render_layout(w, h)` helper
  - `test_wide_terminal_still_shows_hints` — verifies no content loss on 200-col terminal
  - `test_wide_terminal_has_margins` — verifies margin cells are blank at col 10 and col 190
- **Manual test**: `cargo run` in a terminal wider than 100 cols — confirm centred content with visible margins
- **Regression**: all 5 pre-existing layout tests must continue to pass (they use 50–80 col widths, all ≤ 100)

### Notes

- **High-risk item:** The `if area.width > MAX_WIDTH` guard must use strict `>` not `>=`. At exactly `MAX_WIDTH`, `Length(MAX_WIDTH)` exactly fills the terminal — the `Min(0)` margins get zero width, which is valid in ratatui (they render as nothing). Using `>=` would produce the same result but the intent is clearer with `>` (centering only kicks in when there's room for margins).
- **Future consideration:** `MAX_WIDTH` could be added to `Config` alongside `precision`, `max_undo_history`, etc. The struct is already in `src/config/config.rs` and the pattern is established. Not needed now.
- **Known limitation:** The margin areas are truly blank — no background colour is set. On terminals with non-default backgrounds this may look slightly different, but for standard terminal emulators it is invisible.

## Review Notes

- Adversarial review completed
- Findings: 8 total, 7 fixed, 1 skipped (F7 — `full_content` duplication across test modules, Low/Real, deferred)
- Resolution approach: auto-fix
- Fixed: F1 (test comment precision), F2 (margin assumption documented), F3 (min-height guard added), F4 (post-centering comment), F5 (`"[NORMAL]"` assertion), F6 (explicit mode precondition), F8 (`pub(crate)` visibility)
