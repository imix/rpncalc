---
stepsCompleted: [step-01-init, step-01b-continue, step-02-discovery, step-02b-vision, step-02c-executive-summary, step-03-success, step-04-journeys, step-05-domain, step-06-innovation, step-07-project-type, step-08-scoping, step-09-functional, step-10-nonfunctional, step-11-polish, step-12-complete]
inputDocuments: []
workflowType: 'prd'
classification:
  projectType: cli_tool
  domain: general
  complexity: low
  projectContext: greenfield
---

# Product Requirements Document - rpncalc

**Author:** Boss
**Date:** 2026-03-15

## Executive Summary

rpncalc is a keyboard-native RPN calculator for CLI power users — built to live in the terminal, not alongside it. The gap is real: orpie (the closest prior art) is abandonware, `dc` is cryptic and line-based, HP calculators are physical devices or emulators outside the terminal. rpncalc fills this gap as a modern, actively-maintained alternative.

The core promise is two-speed usability: zero friction for everyday arithmetic, zero hunting for rare operations. Both are delivered through a dynamic, context-sensitive hints pane — a live tutorial that teaches the calculator as you use it, without a README.

**Differentiator:** Discoverability is the product, not a feature. Inspired by Helix and Zellij's context-aware keybinding displays, the hints pane adapts to calculator state — showing relevant operations, available constants, and active modes — so users explore confidently. Full-state undo snapshots (stack + registers + mode) make experimentation fearless. Named memory registers with undo semantics make it powerful.

**Classification:** CLI/TUI tool · General productivity domain · Low complexity · Greenfield (existing scaffold is disposable)

## Success Criteria

### User Success

- rpncalc replaces all other calculator tools (browser, phone) for daily workflow
- A typical session (launch → result → exit) completes in under 30 seconds
- Rare operations (e.g., `1/x`, `!`, trig inverses) are found via the hints pane in under 5 seconds, without external documentation
- Users discover new operations through use — the hints pane is the only onboarding required

### Technical Success

- Startup time under 500ms on modern hardware
- No crashes or data loss under normal operation or unexpected termination
- Session state (stack + registers) persists reliably across restarts
- Undo covers all state-mutating operations without exception

### Measurable Outcomes

- User reaches for rpncalc first for every calculation need
- Zero "I couldn't find how to do X" moments within the first week of use
- All 41 MVP operations reachable within 2 hint-pane context states from any starting stack state

## Product Scope

### Phase 1 — MVP

**Philosophy:** Experience MVP — every capability ships complete. No crippled features. The 30-second session must be achievable from day one.

- Interactive TUI: stack pane (HP 48-style), input line, dynamic context-sensitive hints pane
- Full arithmetic, trig, log/exp, and bitwise operations
- Named memory registers (STO/RCL with string identifiers), undo-able
- Full-state undo/redo (stack + registers + modes, depth configurable)
- Session persistence: stack + registers restored on launch (`~/.rpncalc/session.json`)
- Clipboard: copy top-of-stack in current representation style
- Base display modes: DEC/HEX/OCT/BIN with cycleable representation styles
- Angle modes: DEG/RAD/GRAD
- Built-in constants: π, e, φ
- Config file: `~/.rpncalc/config.toml`

### Phase 2 — Growth

- Unit conversion via `~/.rpncalc/units.toml` rules file, surfaced in hints pane
- Shell completion for bash, zsh, fish (operation names, register names, constants)
- Undo tree (vim-style branching history)

### Phase 3 — Vision

- User-configurable keybindings
- Custom operation macros
- Shareable register/session snapshots
- Plugin system for domain-specific operation sets (e.g., electrical engineering, statistics)

### Risk Mitigation

**Technical:** The dynamic hints pane is the most novel implementation challenge — a state machine mapping calculator state to hint categories. Mitigation: implement as a separate, testable module with data-driven config; ship static hints first and layer context-sensitivity incrementally.

**Resource:** Solo developer + AI. Phase 2 features are clean cut lines if scope pressure arises. Hints pane is non-negotiable for MVP; representation style cycling may slip to Phase 2 if needed.

## User Journeys

### Journey 1: The 30-Second Calc (Daily Happy Path)

Mid-morning. Boss is writing a shell script for capacity planning — he needs total memory divided by per-node allocation. He types `rpncalc`, Enter. TUI opens instantly. He types `131072`, Enter, `24576`, Enter, `/`. Result on stack. Clipboard shortcut. Pastes into script. `q`. Done in 22 seconds. Never left the terminal, never touched a mouse.

*Capabilities revealed:* Instant startup, single-keystroke quit, clipboard copy, clean input flow, stack always visible.

### Journey 2: The Rare Operation Discovery (Discovery Path)

Boss needs a factorial for a combinatorics problem. He knows rpncalc can do it — probably — but can't recall the key. He glances at the hints pane. Stack has one value; under "unary ops" he sees `!  factorial`. Types `!`. Done in 4 seconds. No tab switch, no man page, no Google.

*Capabilities revealed:* Context-sensitive hints pane, operation grouping, state-aware hint updates.

### Journey 3: Multi-Step Calculation with Memory (Power Use)

Boss computes resistor networks. He pushes two values, calculates their parallel combination (`1/x swap 1/x + 1/x`), stores as `r1`. Repeats for `r2`. Then `r1 RCL r2 RCL +`. Midway he makes an error — undo twice, stack rewinds cleanly. He closes his laptop. Next morning: `r1` and `r2` are still there.

*Capabilities revealed:* Named registers, undo across all operations including STORE, session persistence.

### Journey 4: Configuration (Setup Path)

Boss wants `kWh → MJ` conversion. Opens `~/.rpncalc/units.toml`, adds one line: `kWh -> MJ: * 3.6`. Restarts rpncalc. Conversion appears in hints pane under "units". Types `500 kWh→MJ`. Done.

*Capabilities revealed:* User-editable rules file, conversions surfaced in hints, format simple enough to edit without documentation.

| Journey | Key Capabilities |
|---|---|
| 30-second daily calc | Fast startup, clipboard, single-keystroke quit |
| Rare op discovery | Context-sensitive hints, state-aware grouping |
| Multi-step with memory | Named registers, undo semantics, persistence |
| Configuration | Rules file, units in hints pane |

## CLI/TUI Specific Requirements

### Architecture

- **Runtime:** Single binary, launched directly from shell
- **TUI framework:** ratatui + crossterm (Rust)
- **Config directory:** `~/.rpncalc/` (XDG config dir respected)
- **Startup sequence:** Load config → restore session → render first frame

### Command Structure

No subcommands. All interaction via TUI after launch. Future: `rpncalc --reset` to clear session state; `rpncalc --completions <shell>` to emit completion script.

### Configuration Schema

```toml
# ~/.rpncalc/config.toml

[display]
angle_mode = "deg"       # deg | rad | grad
base = "dec"             # dec | hex | oct | bin
precision = 15           # decimal places displayed
hex_style = "c"          # c (0xFF) | asm ($FF) | css (#FF) | intel (FFh)

[behaviour]
persist_session = true
max_undo_history = 1000

[units]
rules_file = "~/.rpncalc/units.toml"  # user-defined conversions (Phase 2)
```

Session state: `~/.rpncalc/session.json`
Operation history: `~/.rpncalc/history.toml`

### Output & Representation

- Stack values displayed per active base and precision
- Clipboard copies value in current representation style (base + prefix)
- Representation styles (cycleable via keybinding):
  - Hex: `0xFF` (C) · `$FF` (ASM) · `#FF` (CSS) · `FFh` (Intel)
  - Binary: `0b11111111` or `11111111b`
  - Octal: `0o377` or `0377`
  - Decimal: plain number
- Active representation style shown in status bar
- No stdout output during normal operation

### Shell Completion (Phase 2)

Completion script via `rpncalc --completions <shell>`. Completes: operation names, constant names, register names (dynamic from session). Shells: bash, zsh, fish.

## Functional Requirements

### Stack Operations

- FR1: User can push a numeric value (integer, float, hex, octal, binary) onto the stack
- FR2: User can apply a binary operation to the top two stack values, replacing them with the result
- FR3: User can apply a unary operation to the top stack value, replacing it with the result
- FR4: User can swap the top two stack values
- FR5: User can duplicate the top stack value
- FR6: User can drop (discard) the top stack value
- FR7: User can rotate the top three stack values (rotate-up: X→Y, Y→Z, Z→X)
- FR8: User can clear the entire stack

### Arithmetic & Operations

- FR9: User can perform basic arithmetic: add, subtract, multiply, divide, power, modulo
- FR10: User can perform trigonometric operations: sin, cos, tan, asin, acos, atan
- FR11: User can perform logarithmic/exponential operations: log10, ln, exp, exp10
- FR12: User can perform utility operations: sqrt, square, reciprocal, absolute value, factorial
- FR13: User can perform bitwise operations: AND, OR, XOR, NOT, shift left, shift right
- FR14: User can push built-in constants: π, e, φ

### Display & Modes

- FR15: User can switch the active numeric base (DEC/HEX/OCT/BIN); all stack values redisplay in the new base
- FR16: User can switch the active angle mode (DEG/RAD/GRAD); trig operations use the active mode
- FR17: User can cycle the active representation style for non-decimal bases
- FR18: User can view the full stack at all times with the most recent value at the top

### Discoverability & Help

- FR19: User can view a context-sensitive hints pane showing operations relevant to current stack state
- FR20: The hints pane groups operations by category (arithmetic, trig, stack ops, constants, modes)
- FR21: The hints pane updates immediately on every stack state change
- FR22: User can see active mode indicators (angle mode, base, representation style) in a persistent status bar

### Memory Registers

- FR23: User can store the top stack value into a named register (string identifier)
- FR24: User can recall a named register's value onto the stack
- FR25: User can view all defined registers and their values
- FR26: User can delete a named register

### Undo & History

- FR27: User can undo any state-mutating operation (stack change, register store/delete, mode change)
- FR28: User can redo a previously undone operation
- FR29: Undo and redo restore complete calculator state (stack + registers + modes)

### Session & Persistence

- FR30: The calculator restores previous stack and registers automatically on launch
- FR31: User can enable or disable session persistence via `config.toml`
- FR32: User can reset session state (clear stack, registers, history)

### Clipboard

- FR33: User can copy the top stack value to the system clipboard
- FR34: The copied value uses the current representation style (base + format prefix)

### Configuration

- FR35: User can configure default angle mode, base, precision, and representation style via `config.toml`
- FR36: User can configure undo history depth limit via `config.toml`
- FR37: [Phase 2] User can define custom unit conversion rules in `units.toml`

### Shell Integration

- FR38: [Phase 2] User can generate shell completion scripts for bash, zsh, and fish via `--completions` flag
- FR39: [Phase 2] Shell completions include operation names, constant names, and current session register names
- FR40: User invokes register operations via `<name> STORE` and `<name> RCL` command syntax
- FR41: The hints pane surfaces currently defined register names and their recall commands when registers exist in session

## Non-Functional Requirements

### Performance

- NFR1: Startup time (binary launch to interactive TUI ready) under 500ms on hardware from 2020 or newer
- NFR2: Keypress-to-rendered-result latency under 50ms for all standard operations
- NFR3: Arbitrary-precision operations on values up to 10,000 digits complete within 200ms
- NFR4: TUI redraws complete within one frame interval (≤16ms at 60fps) with no tearing or dropped frames

### Reliability

- NFR5: Session writes are atomic — no corrupt state on interrupted write
- NFR6: Session state survives clean exit and SIGTERM; stack integrity maintained after unexpected process termination (SIGKILL excluded)
- NFR7: All invalid input produces a visible error message and leaves stack unchanged — no panics or crashes
- NFR8: Arithmetic errors (division by zero, domain errors, overflow) are caught and reported without modifying stack state

### Usability

- NFR9: Every MVP operation is reachable via the hints pane without consulting any external documentation
- NFR10: The hints pane always displays at least one relevant operation for any non-empty stack state
- NFR11: A user with no prior rpncalc experience can perform a two-operand arithmetic operation within 60 seconds of first launch
