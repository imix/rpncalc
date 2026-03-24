# Implementation: Push Value

## Behaviour
../usecase.md

## Design Decisions
- Auto-alpha-mode: any digit keypress from normal mode triggers alpha mode
  directly (`Action::AlphaChar(c)`) — no explicit `i` required
- Digit separators (`_`) are stripped before parsing, allowing `1_000_000`
- Float detection is heuristic: presence of `.` or `e`/`E` in input;
  everything else is attempted as arbitrary-precision integer (IBig)
- f64 intermediate used for float parsing before conversion to FBig

## Source Files
- `src/input/parser.rs` — parse_value(): parses string input into
  CalcValue (Integer or Float), handling hex/oct/bin prefixes and
  digit separators
- `src/input/handler.rs` — handle_key(): maps digit keypresses to
  Action::AlphaChar, Enter to Action::AlphaSubmit, Esc to
  Action::AlphaCancel
- `src/input/mode.rs` — AppMode::Alpha state definition
- `src/engine/stack.rs` — CalcState::push(): appends parsed value to stack

## Commits
- 1695d6a feat: complete Epic 1 — Core Engine Foundation
- 7066c63 feat: complete Epics 2–4 + layout width cap

## Tests
- `src/input/parser.rs` (inline `#[cfg(test)]`) — covers all numeric
  formats: integers, floats, hex/oct/bin with both prefix cases, negative
  variants, digit separators, and error cases (empty, garbage, invalid digits)
- `src/engine/stack.rs` (inline `#[cfg(test)]`) — covers push/pop behaviour

## Status
- **State:** complete
- **Created:** 2026-03-21
- **Last verified:** 2026-03-21

## Notes
None
