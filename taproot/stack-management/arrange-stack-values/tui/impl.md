# Implementation: Arrange Stack Values

## Behaviour
../usecase.md

## Design Decisions
- All five ops map to single-key normal-mode bindings: `s`=swap, `p`=dup,
  `d`=drop, `r`=rotate, Enter-with-empty-buffer=dup (HP48 convention)
- Clear has no dedicated key in normal mode — accessible via chord or
  alpha command; Enter in normal mode duplicates rather than clears (HP convention)
- All ops return `Result<>` — underflow surfaces as `CalcError::StackUnderflow`
  which the app layer renders to the ErrorLine

## Source Files
- `src/engine/stack.rs` — CalcState: swap(), dup(), drop(),
  rotate(), clear() — all transactional, return Result
- `src/input/handler.rs` — handle_key(): maps s/p/d/r/Enter
  to Action::Execute(Op::*)
- `src/engine/ops.rs` — Op enum variants and dispatch

## Commits
- 1695d6a feat: complete Epic 1 — Core Engine Foundation
- 7066c63 feat: complete Epics 2–4 + layout width cap

## Tests
- `src/engine/stack.rs` (inline `#[cfg(test)]`) — covers swap, dup, drop,
  rotate, clear including underflow cases and deep-stack invariants

## Status
- **State:** complete
- **Created:** 2026-03-21
- **Last verified:** 2026-03-21

## Notes
None
