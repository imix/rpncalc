# Implementation: Browse Hints Pane

## Behaviour
../usecase.md

## Design Decisions
- HintsPane is purely functional — it takes `&CalcState` and `&AppMode`
  and renders directly; no owned state
- Mode dispatch is a chain of early-return guards: AlphaStore, PrecisionInput,
  Browse, ConvertInput, Insert, InsertUnit, Alpha, Chord each return their own
  compact modal layout; only Normal falls through to the full categorised grid
- Context-sensitivity is implemented via stack depth check: depth 0 shows
  push/constants; depth ≥1 adds unary ops; depth ≥2 adds binary op hints
- Register section rendered only when `CalcState.registers` is non-empty
- Responsive: layout module controls pane visibility — collapses at <60 cols
- `SESSION_OPS` constant (contains `Q quit`) rendered in its own SESSION
  section after chord leaders — `Q` is a session-level action, not a stack
  manipulation op; removing it from `STACK_OPS` makes both sections more scannable
- UNITS section (Normal mode) rendered conditionally: only when `stack.last()`
  is `CalcValue::Tagged`; absent for plain values and empty stack

## Source Files
- `src/tui/widgets/hints_pane.rs` — full hints pane render logic, all
  category tables (ARITHMETIC, STACK_OPS, SESSION_OPS, TRIG_OPS, LOG_OPS,
  FN_OPS, CONST_OPS, ANGLE_OPS, BASE_OPS, HEX_STYLE_OPS, CHORD_LEADERS)
- `src/tui/layout.rs` — responsive layout constraints controlling pane
  width and visibility
- `src/input/mode.rs` — AppMode state machine (Normal/Chord/Alpha)

## Commits
- 7066c63 feat: complete Epics 2–4 + layout width cap
- `84f31d106c43c99f38c5a8fedb608ea15f4552e1` — (auto-linked by taproot link-commits)
- `bc815b4bd1fe63e0e0a3428c0add07d89f6c96b7` — (auto-linked by taproot link-commits)

## Tests
- `src/tui/widgets/hints_pane.rs` — AC-1: `test_depth0_shows_constants_leader`, `test_depth0_shows_stack_ops`; AC-2: `test_depth2_shows_full_arithmetic`, `test_normal_mode_shows_add_op`; AC-3: `test_registers_shows_section_header`, `test_registers_shows_register_name`, `test_registers_shows_recall_command`; AC-5: `test_quit_in_session_section_not_stack`; AC-6: `test_convert_input_mode_shows_header`, `test_convert_input_mode_shows_key_table`, `test_convert_input_mode_hides_normal_sections`; AC-7: see unit-aware-values AC-25 (Insert mode unit hint line); AC-8: `test_units_section_shown_when_top_is_tagged`, `test_units_section_absent_when_top_is_plain`, `test_units_section_absent_when_stack_empty`
- `src/tui/layout.rs` — AC-4: `test_narrow_terminal_hides_hints` (width<60 collapses pane), `test_wide_terminal_shows_hints`, `test_medium_terminal_shows_hints`

## Status
- **State:** complete
- **Created:** 2026-03-21
- **Last verified:** 2026-03-26

## Notes
None
