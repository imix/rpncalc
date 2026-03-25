# Implementation: tui

## Behaviour
../usecase.md

## Design Decisions
- **Label computed in handler, not app**: `command_label(mode, event)` lives in `handler.rs` alongside the key→action mapping. Called before `apply()` so the mode is still `Chord(cat)` when we need the chord leader char to build two-key labels (`rf`, `md`, etc.).
- **Label stored as formatted string in App**: `App.last_command: Option<String>` stores the full `"keys → op"` string. Simpler than storing keys and op separately; the display format is the contract.
- **Centre truncation is omit-or-nothing**: If the centre label would overlap left or right sections, it is omitted entirely. No partial display. Left (`[NORMAL]`) and right (`RAD  DEC`) always take priority. Implemented by computing required widths and skipping the centre span if space is insufficient.
- **`command_label` returns `None` for non-label-updating keys**: Chord leader entry, `InsertChar`, navigation, `AlphaSubmit`, `AlphaCancel`, `EnterStoreMode`, and `BrowseConfirm` all return `None`. Only `Execute(Op)`, `Undo`, `Redo`, `Yank`, `SetAngleMode`, `SetBase`, `SetHexStyle`, and `InsertSubmitThen` return `Some`.
- **`main.rs` sets `last_command` unconditionally when label is `Some`**: Even failed operations update the label (per AC-3). The label is computed from the key event, not the action result.
- **Op name format**: English words from hints pane text (`add`, `floor`, `deg`) — not symbols or operator characters.

## Source Files
- `src/tui/app.rs` — adds `last_command: Option<String>` field
- `src/input/handler.rs` — adds `command_label()` and `op_name()` helper functions
- `src/main.rs` — computes label and sets `app.last_command` in the event loop
- `src/tui/widgets/mode_bar.rs` — renders centre label with omit-on-overflow logic
- `src/tui/layout.rs` — passes `app.last_command.as_deref()` to mode_bar render

## Commits
- placeholder

## Tests
- `src/input/handler.rs` — `command_label` tests: AC-2 (chord two-key label), AC-3 (label returned regardless of stack depth), AC-4 (navigation returns None), AC-7 (InsertSubmitThen returns label), AC-11 (EnterStoreMode returns None)
- `src/tui/widgets/mode_bar.rs` — mode bar render tests: AC-1 (single op label), AC-2 (chord label rendered), AC-5 (undo label), AC-6 (blank on session start), AC-8 (mode/settings not displaced by label), AC-9 (yank label), AC-10 (mode-change chord label), overflow truncation (label omitted when too narrow)

## Status
- **State:** in-progress
- **Created:** 2026-03-25
- **Last verified:** 2026-03-25

## Notes
None
