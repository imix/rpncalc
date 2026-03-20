# Story 3.4: Context-Sensitive Hints Filtering

Status: done

## Story

As a CLI power user,
I want the hints pane to show only operations relevant to the current stack depth,
So that I'm never shown operations I can't yet use, and push/constant hints are prominent on an empty stack.

## Acceptance Criteria

1. **Given** the stack is empty (depth 0), **When** the hints pane renders, **Then** NO arithmetic ops (binary or unary) are shown **And** push/constants chord leaders ARE shown.

2. **Given** the stack has exactly one value (depth 1), **When** the hints pane renders, **Then** unary ops (`!`, `n`) are shown **And** binary ops (`+`, `-`, `*`, `/`, `^`, `%`) are NOT shown.

3. **Given** the stack has two or more values (depth ≥ 2), **When** the hints pane renders, **Then** all arithmetic ops (binary + unary) are shown alongside stack ops and all chord leaders.

4. **Given** any stack mutation (push, drop, execute, undo), **When** the next render occurs, **Then** hints immediately reflect the new depth — no extra state management needed (driven by `&CalcState` on each render call).

5. **Given** the stack is non-empty (depth ≥ 1), **When** the hints pane renders, **Then** at least one arithmetic op is always visible.

## Tasks / Subtasks

- [x] Task 1: Add depth-filtering constants to `src/tui/widgets/hints_pane.rs`
  - [x] Add `UNARY_OPS` constant (ops valid at depth 1: `!` fact, `n` neg)
  - [x] Add `CHORD_LEADERS_DEPTH0` constant (leaders valid at depth 0: `c` const, `m` mode, `x` base, `X` hex)

- [x] Task 2: Update Normal-mode render in `hints_pane::render` (AC: 1–5)
  - [x] Rename `_state` parameter to `state` (remove underscore — it will now be used)
  - [x] Read `let depth = state.stack.len();` at top of Normal mode block
  - [x] depth 0: no ARITHMETIC section; show STACK section; show `CHORD_LEADERS_DEPTH0`
  - [x] depth 1: ARITHMETIC header + `UNARY_OPS` only; show STACK section; show all `CHORD_LEADERS`
  - [x] depth ≥ 2: full ARITHMETIC (all 8 ops); show STACK section; show all `CHORD_LEADERS`

- [x] Task 3: Update test helper and fix regressions in `hints_pane.rs` tests
  - [x] Update `render_hints` helper signature: add `state: CalcState` parameter (before width/height)
  - [x] Update `render_hints` body: pass `&state` to `render(...)` instead of creating internal `CalcState::new()`
  - [x] Fix `test_normal_mode_shows_arithmetic_header`: pass state with 2 values so depth ≥ 2
  - [x] Fix `test_normal_mode_shows_add_op`: pass state with 2 values so depth ≥ 2
  - [x] Fix ALL other existing tests that call `render_hints` in Normal mode to pass a `CalcState` argument

- [x] Task 4: Add new depth-filtering tests (AC: 1–5)
  - [x] `test_depth0_hides_arithmetic_header` — empty state: `!content.contains("ARITHMETIC")`
  - [x] `test_depth0_hides_binary_ops` — empty state: `!content.contains("+")` and `!content.contains("add")`
  - [x] `test_depth0_hides_unary_ops` — empty state: `!content.contains("fact")` and `!content.contains("neg")`
  - [x] `test_depth0_shows_constants_leader` — empty state: `content.contains("const")`
  - [x] `test_depth0_shows_stack_ops` — empty state: `content.contains("STACK")`
  - [x] `test_depth0_hides_trig_leader` — empty state: `!content.contains("trig")`
  - [x] `test_depth1_shows_unary_ops` — 1-value state: `content.contains("fact")` and `content.contains("neg")`
  - [x] `test_depth1_hides_binary_ops` — 1-value state: `!content.contains("add")` and `!content.contains("sub")`
  - [x] `test_depth1_shows_all_chord_leaders` — 1-value state: `content.contains("trig")`
  - [x] `test_depth2_shows_full_arithmetic` — 2-value state: `content.contains("add")` and `content.contains("mul")`

- [x] Task 5: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — 250 tests pass (240 pre-existing + 10 new)

## Dev Notes

### One File Changes

Only `src/tui/widgets/hints_pane.rs` changes. No other production or test files.

### New Constants

Add after `CHORD_LEADERS`:

```rust
const UNARY_OPS: &[(&str, &str)] = &[("!", "fact"), ("n", "neg")];

const CHORD_LEADERS_DEPTH0: &[(&str, &str)] = &[
    ("c", "const"),
    ("m", "mode"),
    ("x", "base"),
    ("X", "hex"),
];
```

`UNARY_OPS` is the depth-1 subset of `ARITHMETIC`. The four depth-0 leaders are the ones that don't require stack values — `c`=push constants, `m`=angle mode, `x`=numeric base, `X`=hex style. The value-consuming leaders (`t`=trig, `l`=log, `f`=fn) are hidden at depth 0.

### Updated Render Signature

```rust
// BEFORE:
pub fn render(f: &mut Frame, area: Rect, mode: &AppMode, _state: &CalcState) {

// AFTER:
pub fn render(f: &mut Frame, area: Rect, mode: &AppMode, state: &CalcState) {
```

The Alpha and Chord branches are UNCHANGED — they don't look at `state`. Only the Normal mode block uses depth.

### Normal Mode Block — Full Replacement

Replace the existing Normal mode block (everything after the Chord `return;`):

```rust
let depth = state.stack.len();
let dim = Style::default().add_modifier(Modifier::DIM);
let mut lines: Vec<Line<'static>> = Vec::new();

if depth >= 2 {
    lines.push(Line::styled("ARITHMETIC", dim));
    lines.extend(entries_to_lines(ARITHMETIC));
    lines.push(Line::raw(""));
} else if depth == 1 {
    lines.push(Line::styled("ARITHMETIC", dim));
    lines.extend(entries_to_lines(UNARY_OPS));
    lines.push(Line::raw(""));
}

lines.push(Line::styled("STACK", dim));
lines.extend(entries_to_lines(STACK_OPS));
lines.push(Line::raw(""));

if depth == 0 {
    lines.extend(chord_leaders_to_lines(CHORD_LEADERS_DEPTH0));
} else {
    lines.extend(chord_leaders_to_lines(CHORD_LEADERS));
}

f.render_widget(Paragraph::new(lines), area);
```

Note: `dim` is declared in this block — the existing `let dim = Style::default().add_modifier(Modifier::DIM);` in the Chord branch is a separate declaration in that block's scope. No conflict.

### CRITICAL: Test Helper Must Accept State

The existing `render_hints` helper creates `CalcState::new()` internally:

```rust
// CURRENT (BROKEN after story implementation):
fn render_hints(mode: AppMode, width: u16, height: u16) -> ratatui::buffer::Buffer {
    let state = CalcState::new();
    ...
    terminal.draw(|f| render(f, f.area(), &mode, &state)).unwrap();
    ...
}
```

After this story, empty state = depth 0, so ARITHMETIC won't render. Two existing tests will fail:
- `test_normal_mode_shows_arithmetic_header` — asserts `content.contains("ARITHMETIC")`
- `test_normal_mode_shows_add_op` — asserts `content.contains("+")` and `content.contains("add")`

**Fix:** Update helper signature and fix those two tests:

```rust
fn render_hints(mode: AppMode, state: CalcState, width: u16, height: u16) -> ratatui::buffer::Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| render(f, f.area(), &mode, &state))
        .unwrap();
    terminal.backend().buffer().clone()
}
```

All call sites that pass `AppMode::Normal` need a `CalcState`. A helper to create a state with N values:

```rust
fn state_with_depth(n: usize) -> CalcState {
    let mut s = CalcState::new();
    for i in 0..n {
        s.stack.push(CalcValue::from_f64(i as f64 + 1.0));
    }
    s
}
```

Updated broken tests:

```rust
#[test]
fn test_normal_mode_shows_arithmetic_header() {
    let buf = render_hints(AppMode::Normal, state_with_depth(2), 40, 15);
    let content = full_content(&buf);
    assert!(content.contains("ARITHMETIC"));
}

#[test]
fn test_normal_mode_shows_add_op() {
    let buf = render_hints(AppMode::Normal, state_with_depth(2), 40, 15);
    let content = full_content(&buf);
    assert!(content.contains('+'));
    assert!(content.contains("add"));
}
```

Tests that can stay with empty state (not affected):
- `test_normal_mode_shows_stack_header` — STACK always shows
- `test_normal_mode_shows_chord_leaders` — chord leaders (`›`) always show
- `test_narrow_pane_no_panic` — just checks no panic

Those three need their call sites updated to pass `CalcState::new()` as the state arg.

### CalcValue Import for Tests

The test module already imports from `use crate::engine::stack::CalcState;`. Add:

```rust
use crate::engine::value::CalcValue;
```

This is needed for `CalcValue::from_f64(...)` in `state_with_depth`.

### Depth Is Reactive — No Extra State

`hints_pane::render` receives `&CalcState` fresh on every frame from `layout.rs`:

```rust
// layout.rs line 19 — already wired:
hints_pane::render(f, inner[1], &app.mode, &app.state);
```

No caching, no event handling, no explicit notifications. Depth changes automatically reflect on next render. AC4 is satisfied architecturally.

### Architecture Compliance

- Only `src/tui/widgets/hints_pane.rs` changes — no new files [Source: architecture.md#Test location]
- `hints_pane::render` remains purely functional — reads `&AppMode` and `&CalcState` only, no side effects [Source: architecture.md#Widgets]
- No `unwrap()` in non-test code — `state.stack.len()` is infallible [Source: architecture.md#No unwrap() policy]
- `CalcState` is the sole mutable state — tests verify rendering by inspecting buffer content, not state [Source: architecture.md#Calculator State Ownership]

### Previous Story Learnings (Story 3.3)

- `cargo fmt` reformats multi-arg assert macros to multi-line style — run `cargo fmt` last
- Test count was 240 after Story 3.3; this story adds ~10 tests → expect ~250 total
- No production code beyond `hints_pane.rs` needed — layout.rs already passes `&app.state`

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — clean implementation, no issues.

### Completion Notes List

- Renamed `_state` → `state` in render signature; depth read as `state.stack.len()` on every render
- Added `UNARY_OPS` (`!`, `n`) and `CHORD_LEADERS_DEPTH0` (c/m/x/X) constants
- Normal mode block replaced with depth-conditional logic: depth 0 shows STACK + 4 chord leaders; depth 1 shows ARITHMETIC header + UNARY_OPS + STACK + all chord leaders; depth ≥ 2 shows full ARITHMETIC + STACK + all chord leaders
- `render_hints` test helper updated to accept `state: CalcState` parameter; `state_with_depth(n)` helper added
- `CalcValue` import added to test module
- Fixed two regressions: `test_normal_mode_shows_arithmetic_header` and `test_normal_mode_shows_add_op` now pass `state_with_depth(2)`
- All other Normal-mode tests updated to pass explicit `CalcState::new()` or `state_with_depth(n)`
- 10 new depth-filtering tests added covering all 5 ACs
- `cargo fmt` reformatted `render_hints` signature and chord test call sites to multi-line style
- 250 tests pass total (240 pre-existing + 10 new)

### File List

- `src/tui/widgets/hints_pane.rs` — depth-conditional render logic + new constants + updated test helper + 10 new tests
