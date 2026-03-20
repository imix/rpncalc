# Story 3.5: Register Section in Hints Pane

Status: done

## Story

As a CLI power user,
I want the hints pane to show my defined registers and how to recall them,
So that I can see and access my named values without having to remember their names.

## Acceptance Criteria

1. **Given** one or more named registers are defined, **When** the hints pane renders in Normal mode, **Then** a `REGISTERS` section appears showing each register name, its current value, and the recall command (e.g. `r1 RCL`).

2. **Given** no registers are defined, **When** the hints pane renders, **Then** no register section appears — no `REGISTERS` heading, no empty placeholder, no visual noise.

3. **Given** a register is stored or deleted, **When** the hints pane next renders, **Then** the register section immediately reflects the updated register list (reactive — no extra state management needed).

## Tasks / Subtasks

- [x] Task 1: Add `registers_to_lines` helper to `src/tui/widgets/hints_pane.rs`
  - [x] Sort registers by key name (`sort_by` on iterated `HashMap` entries) for consistent display order
  - [x] Format each entry as `Line::raw(format!("{name}  {val_str}  {name} RCL"))` using `val.display_with_base(state.base)`
  - [x] Return `Vec<Line<'static>>` — all strings are owned via `format!`, so no lifetime issue

- [x] Task 2: Add conditional REGISTERS section to Normal-mode render block
  - [x] After chord leaders block: `if !state.registers.is_empty() { ... }`
  - [x] Add blank line separator, then `Line::styled("REGISTERS", dim)` header
  - [x] Extend lines with `registers_to_lines(state)`
  - [x] Confirm `dim` is in scope (it is — declared at top of Normal mode block, line 125)

- [x] Task 3: Add tests (AC: 1–3)
  - [x] `test_no_registers_hides_section` — `CalcState::new()` → `!content.contains("REGISTERS")`
  - [x] `test_registers_shows_section_header` — state with one register → `content.contains("REGISTERS")`
  - [x] `test_registers_shows_register_name` — register "r1" → `content.contains("r1")`
  - [x] `test_registers_shows_recall_command` — register "r1" → `content.contains("r1 RCL")`
  - [x] `test_registers_not_shown_in_alpha_mode` — `AppMode::Alpha` + registers → `!content.contains("REGISTERS")`
  - [x] `test_multiple_registers_all_shown` — two registers "aa", "bb" → both appear in content

- [x] Task 4: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — all pre-existing 250 tests pass + 6 new = 256 total

## Dev Notes

### One File Changes

Only `src/tui/widgets/hints_pane.rs` changes. No other production or test files.

### No New Production Imports Needed

The existing production imports are sufficient:
```rust
use crate::engine::stack::CalcState;
use crate::input::mode::{AppMode, ChordCategory};
```

`registers_to_lines(state: &CalcState)` accesses `state.registers` (a `HashMap<String, CalcValue>`) and calls `val.display_with_base(state.base)` — Rust resolves these methods through the type system without needing explicit `use` statements for `CalcValue`, `Base`, or `HashMap`.

### `registers_to_lines` Implementation

Add after `chord_leaders_to_lines`:

```rust
fn registers_to_lines(state: &CalcState) -> Vec<Line<'static>> {
    let mut entries: Vec<_> = state.registers.iter().collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    entries
        .into_iter()
        .map(|(name, val)| {
            let val_str = val.display_with_base(state.base);
            Line::raw(format!("{name}  {val_str}  {name} RCL"))
        })
        .collect()
}
```

Key points:
- `sort_by` on a `Vec<(&String, &CalcValue)>` — `a.cmp(b)` where both are `&&String` which deref to `&String` which is `Ord` ✓
- `format!("{name}  {val_str}  {name} RCL")` creates an owned `String` → `Line::raw(String)` yields `Line<'static>` (Cow::Owned) ✓
- No truncation — Story 3.6 handles layout adaptation for narrow terminals

### Normal Mode Block — Addition

The registers section slots in at the very end of the Normal mode block, after chord leaders:

```rust
    if depth == 0 {
        lines.extend(chord_leaders_to_lines(CHORD_LEADERS_DEPTH0));
    } else {
        lines.extend(chord_leaders_to_lines(CHORD_LEADERS));
    }

    // ← ADD THIS BLOCK:
    if !state.registers.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::styled("REGISTERS", dim));
        lines.extend(registers_to_lines(state));
    }

    f.render_widget(Paragraph::new(lines), area);
```

`dim` is in scope here — it was declared at line 125: `let dim = Style::default().add_modifier(Modifier::DIM);`

### REGISTERS Section Is Normal Mode Only

Alpha and Chord branches both `return;` before reaching the Normal mode block. So:
- Alpha mode → early return → no REGISTERS section (correct — user is typing)
- Chord mode → early return → no REGISTERS section (correct — submenu is active)
- Normal mode → reaches register block → shows if non-empty (correct)

No special handling needed for other modes.

### Test Pattern for States with Registers

The test module already imports `CalcValue`. Insert registers directly into the state:

```rust
let mut s = CalcState::new();
s.registers.insert("r1".to_string(), CalcValue::from_f64(3.14));
// Then pass s to render_hints(AppMode::Normal, s, width, height)
```

`CalcState.registers` is `pub` (required by architecture for serialization/deserialization access).

### AC3 Is Reactive by Architecture

`hints_pane::render` receives `&CalcState` fresh every frame from `layout.rs`. Any change to `state.registers` via `App::apply()` is immediately visible on the next render. No pub/sub, no event tracking needed. This is the same pattern as depth-reactive filtering in Story 3.4.

### Existing Tests Are Unaffected

All 250 existing tests use `CalcState::new()` or `state_with_depth(n)` — neither populates `state.registers`. Therefore `state.registers.is_empty()` is `true` in every existing test, and the REGISTERS block is skipped entirely. Zero regressions. ✓

### Display Format Details

Line format: `"{name}  {val_str}  {name} RCL"`

Example at DEC base with register `r1 = 3.14`:
```
r1  3.14  r1 RCL
```

Example with longer name `tax = 0.07`:
```
tax  0.07  tax RCL
```

The `display_with_base(base)` method on `CalcValue` already handles base-specific formatting (hex, oct, bin) — if HEX base is active and the register holds an integer, it displays as `0xFF` (or whatever hex style is active). No extra logic needed.

### Architecture Compliance

- Only `src/tui/widgets/hints_pane.rs` changes — no new files [Source: architecture.md#Test location]
- `hints_pane::render` remains purely functional — reads `&AppMode` and `&CalcState` only [Source: architecture.md#Widgets]
- No `unwrap()` in production code — `sort_by` and `format!` are infallible [Source: architecture.md#No unwrap() policy]
- `CalcState` is the sole mutable state — tests insert into `state.registers` directly, verify via buffer content [Source: architecture.md#Calculator State Ownership]

### Previous Story Learnings (Story 3.4)

- `cargo fmt` reformats longer function signatures to multi-line style — run last
- Test count was 250 after Story 3.4; this story adds 6 → expect 256 total
- `render_hints` helper accepts `state: CalcState` as second param — tests must pass a state
- `state_with_depth(n)` helper exists for depth-specific tests; register tests build their own states inline
- `CalcValue` is imported in test module (`use crate::engine::value::CalcValue;`)

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — clean implementation, no issues.

### Completion Notes List

- Added `registers_to_lines(state: &CalcState)` — sorts register entries by name, formats each as `"{name}  {val_str}  {name} RCL"` using `val.display_with_base(state.base)`
- REGISTERS block added after chord leaders in Normal mode: guarded by `!state.registers.is_empty()`, with blank separator and dim header
- Alpha and Chord modes unaffected — both return early before the Normal mode block
- No new production imports needed — method calls resolve through the type system
- `cargo fmt` reformatted `s.registers.insert(...)` calls to two-line style in tests
- 6 new tests added covering all 3 ACs; 256 tests pass total (250 pre-existing + 6 new)

### File List

- `src/tui/widgets/hints_pane.rs` — `registers_to_lines` function + REGISTERS section in Normal mode + 6 new tests
