---
stepsCompleted: [step-01-validate-prerequisites, step-02-design-epics, step-03-create-stories, step-04-final-validation]
inputDocuments: [prd.md, architecture.md, ux-design-specification.md]
---

# rpncalc - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for rpncalc, decomposing the requirements from the PRD, UX Design, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: User can push a numeric value (integer, float, hex, octal, binary) onto the stack
FR2: User can apply a binary operation to the top two stack values, replacing them with the result
FR3: User can apply a unary operation to the top stack value, replacing it with the result
FR4: User can swap the top two stack values
FR5: User can duplicate the top stack value
FR6: User can drop (discard) the top stack value
FR7: User can rotate the top three stack values (rotate-up: X→Y, Y→Z, Z→X)
FR8: User can clear the entire stack
FR9: User can perform basic arithmetic: add, subtract, multiply, divide, power, modulo
FR10: User can perform trigonometric operations: sin, cos, tan, asin, acos, atan
FR11: User can perform logarithmic/exponential operations: log10, ln, exp, exp10
FR12: User can perform utility operations: sqrt, square, reciprocal, absolute value, factorial
FR13: User can perform bitwise operations: AND, OR, XOR, NOT, shift left, shift right
FR14: User can push built-in constants: π, e, φ
FR15: User can switch the active numeric base (DEC/HEX/OCT/BIN); all stack values redisplay in the new base
FR16: User can switch the active angle mode (DEG/RAD/GRAD); trig operations use the active mode
FR17: User can cycle the active representation style for non-decimal bases
FR18: User can view the full stack at all times with the most recent value at the top
FR19: User can view a context-sensitive hints pane showing operations relevant to current stack state
FR20: The hints pane groups operations by category (arithmetic, trig, stack ops, constants, modes)
FR21: The hints pane updates immediately on every stack state change
FR22: User can see active mode indicators (angle mode, base, representation style) in a persistent status bar
FR23: User can store the top stack value into a named register (string identifier)
FR24: User can recall a named register's value onto the stack
FR25: User can view all defined registers and their values
FR26: User can delete a named register
FR27: User can undo any state-mutating operation (stack change, register store/delete, mode change)
FR28: User can redo a previously undone operation
FR29: Undo and redo restore complete calculator state (stack + registers + modes)
FR30: The calculator restores previous stack and registers automatically on launch
FR31: User can enable or disable session persistence via config.toml
FR32: User can reset session state (clear stack, registers, history)
FR33: User can copy the top stack value to the system clipboard
FR34: The copied value uses the current representation style (base + format prefix)
FR35: User can configure default angle mode, base, precision, and representation style via config.toml
FR36: User can configure undo history depth limit via config.toml
FR37: [Phase 2] User can define custom unit conversion rules in units.toml
FR38: [Phase 2] User can generate shell completion scripts for bash, zsh, and fish via --completions flag
FR39: [Phase 2] Shell completions include operation names, constant names, and current session register names
FR40: User invokes register operations via `<name> STORE` and `<name> RCL` command syntax
FR41: The hints pane surfaces currently defined register names and their recall commands when registers exist in session

### NonFunctional Requirements

NFR1: Startup time (binary launch to interactive TUI ready) under 500ms on hardware from 2020 or newer
NFR2: Keypress-to-rendered-result latency under 50ms for all standard operations
NFR3: Arbitrary-precision operations on values up to 10,000 digits complete within 200ms
NFR4: TUI redraws complete within one frame interval (≤16ms at 60fps) with no tearing or dropped frames
NFR5: Session writes are atomic — no corrupt state on interrupted write
NFR6: Session state survives clean exit and SIGTERM; stack integrity maintained after unexpected process termination (SIGKILL excluded)
NFR7: All invalid input produces a visible error message and leaves stack unchanged — no panics or crashes
NFR8: Arithmetic errors (division by zero, domain errors, overflow) are caught and reported without modifying stack state
NFR9: Every MVP operation is reachable via the hints pane without consulting any external documentation
NFR10: The hints pane always displays at least one relevant operation for any non-empty stack state
NFR11: A user with no prior rpncalc experience can perform a two-operand arithmetic operation within 60 seconds of first launch

### Additional Requirements

- **Scaffold reorganization (Story 1 prerequisite):** Existing repo scaffold uses a different structure (string-based op dispatch, no registers.rs, no undo.rs). Story 1 must reorganize the scaffold to match the target module structure before building new features.
- **Starter:** `cargo new rpncalc --bin` approach — single binary crate, flat module structure under src/
- **Core dependencies to pin in Cargo.toml:** ratatui ~0.29, crossterm ~0.28, dashu ~0.4, serde + serde_json 1.x, toml ~0.8, dirs ~5.0, arboard ~3.x, signal-hook ~0.3
- **Module structure must be followed exactly:** engine/, input/, tui/, config/ under src/ — no new top-level modules without architectural review
- **Implementation sequence constraint:** CalcState + CalcValue structs defined first → Action enum → engine ops + tests → handle_key() + tests → session serialization tests → SIGTERM handler
- **Atomic session writes:** write-to-temp → rename pattern in config/session.rs; no partial writes
- **SIGTERM handling:** signal-hook handler in main.rs calls session::save(&state) directly before exit
- **Quality gates:** Every task requires `cargo clippy -- -D warnings` and `cargo fmt` to pass; no unwrap() in non-test code without `// SAFETY:` comment
- **Test placement:** Unit tests co-located in #[cfg(test)] at bottom of each file; integration tests in tests/ directory only
- **CalcState is the sole mutable state** — never introduce secondary state stores; snapshot before execute for undo correctness

### UX Design Requirements

UX-DR1: Implement two-mode input model: Normal mode (immediate single-key execution, no Enter required) and Alpha mode (text buffer with Enter to commit, Esc to cancel)
UX-DR2: Implement auto-alpha-mode entry on any digit keypress from normal mode — digit triggers alpha mode automatically without requiring `i`
UX-DR3: Implement chord leader system: 7 chord leaders (t=trig, l=log/exp, f=functions, c=constants, m=angle-mode, x=base, X=hex-style) with second-key submenu dispatch; Esc cancels mid-chord
UX-DR4: Implement HintsPane state machine with three states: normal (categorized keybinding grid), chord-active (category header + submenu body), alpha-mode (alpha navigation hints); state driven by CalcState + AppMode on every render tick
UX-DR5: Implement context-sensitive hints filtering by stack depth: empty stack shows push/constants hints; one value adds unary ops; two+ values highlights binary ops
UX-DR6: Implement register section in HintsPane — show defined register names and their RCL commands when registers are present; omit the section entirely when no registers defined
UX-DR7: Implement Direction A four-region layout: horizontal split (StackPane left ~40% / HintsPane right ~60%) + InputLine (1 row full width) + ErrorLine (1 row full width) + ModeBar (1 row full width)
UX-DR8: Implement StackPane: numbered rows (X/Y/Z/T then 5/6/…), most-recent value at bottom (X), values right-aligned in current base/representation, X row bolded/accented, truncation with `…`
UX-DR9: Implement InputLine: shows `> ` prompt only in normal mode; shows `> ` + buffer text + blinking cursor in alpha mode
UX-DR10: Implement ErrorLine: fully blank row in normal state; error text in error color on failure; clears automatically on next valid action — never shows success feedback
UX-DR11: Implement ModeBar: left side `[NORMAL]`/`[INSERT]`; right side angle mode (DEG/RAD/GRAD) + base (DEC/HEX/OCT/BIN) + hex style (0xFF/$FF/#FF/FFh — shown only in HEX mode)
UX-DR12: Implement responsive layout breakpoints: ≥80 cols = full layout; 60–79 cols = hints pane narrows + labels may abbreviate; <60 cols = hints pane collapses entirely; ≥24 rows = full stack; 20–23 rows = reduced stack rows; <20 rows = minimum 4 stack rows preserved
UX-DR13: Implement semantic color system (no hardcoded RGB or ANSI codes): roles fg, bg, accent (cyan), ok (green), warn (yellow), error (red), dim, bold — resolved against terminal palette at render time
UX-DR14: Implement Esc as universal cancel: from alpha mode → returns to normal mode and clears buffer; from chord-wait → cancels chord, returns to normal; from normal mode → no-op
UX-DR15: Implement complete normal-mode keybinding table: s=swap, d=drop, p=dup, r=rotate, n=negate, u=undo, Ctrl-R=redo, y=yank, +/-/*/÷/^/%/! = immediate binary/unary ops, Enter=dup (HP convention when buffer empty), q=quit, i=enter alpha mode

### FR Coverage Map

| FR | Epic | Summary |
|---|---|---|
| FR1–8 | Epic 1 | Stack operations (push, binary, unary, swap, dup, drop, rotate, clear) |
| FR9–13 | Epic 1 | All operations (arithmetic, trig, log, utility, bitwise) |
| FR14 | Epic 1 | Built-in constants (π, e, φ) |
| FR15 | Epic 3 | Base switching (DEC/HEX/OCT/BIN) via chord |
| FR16 | Epic 3 | Angle mode switching (DEG/RAD/GRAD) via chord |
| FR17 | Epic 3 | Representation style cycling via chord |
| FR18 | Epic 2 | Full stack always visible |
| FR19–21 | Epic 3 | Context-sensitive hints pane, grouping, live updates |
| FR22 | Epic 2 | Mode indicators in persistent status bar |
| FR23–26 | Epic 4 | Named registers (store, recall, view, delete) |
| FR27–29 | Epic 4 | Full-state undo/redo |
| FR30–32 | Epic 4 | Session persistence (save/restore/reset) |
| FR33–34 | Epic 2 | Clipboard copy with representation style |
| FR35–36 | Epic 4 | Config file (defaults + undo depth) |
| FR37–39 | Phase 2 | Unit conversions, shell completions (deferred) |
| FR40 | Epic 1 | STORE/RCL command syntax parsing |
| FR41 | Epic 3 | Register names surfaced in hints pane |

## Epic List

### Epic 1: Core Engine Foundation
Users gain a complete, fully-tested calculator engine — all data types, stack operations, and computations work correctly, giving the TUI a rock-solid foundation.
**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR6, FR7, FR8, FR9, FR10, FR11, FR12, FR13, FR14, FR40

### Epic 2: Interactive TUI — The 30-Second Calc
Users can launch rpncalc, enter numbers, perform all operations, see the full stack, copy results, and quit — the core 30-second session is fully achievable.
**FRs covered:** FR18, FR22, FR33, FR34

### Epic 3: Discoverability — Hints Pane & Chord System
Users can discover and access every MVP operation through the context-sensitive hints pane and chord system, without consulting any external documentation.
**FRs covered:** FR15, FR16, FR17, FR19, FR20, FR21, FR41

### Epic 4: Power User Features — Registers, Undo & Session
Users can store and recall named values, undo any mistake fearlessly, and trust that calculator state persists reliably across restarts.
**FRs covered:** FR23, FR24, FR25, FR26, FR27, FR28, FR29, FR30, FR31, FR32, FR35, FR36

---

## Epic 1: Core Engine Foundation

Users gain a complete, fully-tested calculator engine — all data types, stack operations, and computations work correctly, giving the TUI a rock-solid foundation.

### Story 1.1: Project Scaffold & Module Structure

As a developer,
I want the project scaffold organized to match the target architecture with all dependencies in place,
So that all subsequent stories have the correct structure and libraries to build upon.

**Acceptance Criteria:**

**Given** the scaffold setup is complete
**When** the project is built
**Then** the build succeeds with no errors or warnings
**And** the module structure matches the architecture document

**Dev Notes:** Run `cargo clippy -- -D warnings` and `cargo fmt`. Add all 8 dependencies from architecture doc.

---

### Story 1.2: Core Data Types

As a developer,
I want the foundational data types defined,
So that stack values, calculator state, errors, and user actions all have a consistent, well-typed representation.

**Acceptance Criteria:**

**Given** the core types are defined
**When** a numeric value is represented
**Then** it can be either an arbitrary-precision integer or an arbitrary-precision float

**Given** the calculator state type exists
**When** it is created with defaults
**Then** it holds a stack of values, a set of named registers, an active angle mode, an active base, and an active hex style

**Given** a calculator error occurs
**When** the error is reported
**Then** it carries a meaningful description (e.g. stack underflow, division by zero, domain error)

**Given** the action type is defined
**When** any user-triggered event occurs
**Then** it is representable as a typed action with no string-based dispatch

---

### Story 1.3: Stack Operations

As a CLI power user,
I want to push values onto the stack and manipulate them with fundamental stack operations,
So that I can build up operands and manage stack depth during calculations.

**Acceptance Criteria:**

**Given** a value is pushed onto the stack
**When** the stack is inspected
**Then** the new value is at the top and the previous contents are preserved below it

**Given** two or more values on the stack
**When** swap is applied
**Then** the top two values exchange positions and stack depth is unchanged

**Given** one or more values on the stack
**When** dup is applied
**Then** the top value is duplicated and stack depth increases by one

**Given** one or more values on the stack
**When** drop is applied
**Then** the top value is removed and stack depth decreases by one

**Given** three or more values on the stack
**When** rotate is applied
**Then** the top three values cycle: the top moves to third position, second moves to top, third moves to second

**Given** any stack
**When** clear is applied
**Then** the stack is empty

**Given** an insufficient stack for an operation (e.g. swap with one item, rotate with two)
**When** the operation is attempted
**Then** the stack is completely unchanged and an error is produced

---

### Story 1.4: All Operations & Built-in Constants

As a CLI power user,
I want to apply arithmetic, trig, log/exp, utility, and bitwise operations, and push built-in constants,
So that I can perform every calculation the calculator offers.

**Acceptance Criteria:**

**Given** two values on the stack
**When** a binary arithmetic operation (add, subtract, multiply, divide, power, modulo) is applied
**Then** the result replaces the top two values and stack depth decreases by one

**Given** a division by zero is attempted
**When** divide is applied
**Then** the stack is unchanged and an appropriate error is produced

**Given** one value on the stack with DEG angle mode active
**When** sin, cos, or tan is applied
**Then** the result treats the input as degrees

**Given** one value on the stack
**When** an inverse trig operation (asin, acos, atan) is applied
**Then** the result is expressed in the active angle unit

**Given** one value on the stack
**When** ln, log10, exp, or exp10 is applied
**Then** the mathematically correct result is pushed

**Given** one value on the stack
**When** sqrt, square, reciprocal, absolute value, or factorial is applied
**Then** the mathematically correct result is pushed

**Given** an operation with no valid result (e.g. sqrt of a negative, ln of zero)
**When** the operation is attempted
**Then** the stack is unchanged and a domain error is produced

**Given** an integer on the stack (two integers for binary bitwise)
**When** a bitwise operation (AND, OR, XOR, NOT, shift left, shift right) is applied
**Then** the correct bitwise result is pushed

**Given** a non-integer is passed to a bitwise operation
**When** the operation is attempted
**Then** the stack is unchanged and an error is produced

**When** a built-in constant (π, e, φ) is pushed
**Then** the constant's value appears on top of the stack

---

### Story 1.5: Numeric Input Parsing & Register Command Syntax

As a CLI power user,
I want to enter numbers in decimal, hex, octal, and binary formats, and use STORE/RCL command syntax,
So that I can work with any numeric representation and invoke named register operations naturally.

**Acceptance Criteria:**

**Given** a decimal integer is entered (e.g. `42`, `-17`)
**When** it is parsed
**Then** it is accepted as a valid integer value

**Given** a decimal float is entered (e.g. `3.14`, `1.5e-3`)
**When** it is parsed
**Then** it is accepted as a valid float value

**Given** a hex value is entered (e.g. `0xFF`, `0xff`)
**When** it is parsed
**Then** it is accepted as the correct integer value

**Given** an octal value is entered (e.g. `0o377`)
**When** it is parsed
**Then** it is accepted as the correct integer value

**Given** a binary value is entered (e.g. `0b11111111`)
**When** it is parsed
**Then** it is accepted as the correct integer value

**Given** a register store command is entered (e.g. `myvar STORE`)
**When** it is parsed
**Then** it is recognized as a store action targeting that register name

**Given** a register recall command is entered (e.g. `r1 RCL`)
**When** it is parsed
**Then** it is recognized as a recall action for that register name

**Given** unrecognizable input is entered
**When** it is parsed
**Then** an invalid input error is produced

---

## Epic 2: Interactive TUI — The 30-Second Calc

Users can launch rpncalc, enter numbers, perform operations, see the full stack, copy results, and quit — the core 30-second session fully achievable.

### Story 2.1: TUI Infrastructure — Event Loop & Layout Shell

As a CLI power user,
I want rpncalc to launch instantly as a full-screen TUI and quit cleanly,
So that it feels like a native terminal instrument from the first keystroke.

**Acceptance Criteria:**

**Given** the binary is executed
**When** the TUI launches
**Then** the screen switches to an alternate terminal buffer and the layout is visible within 500ms

**Given** the TUI is running
**When** the terminal is resized
**Then** the layout reflows to fit the new dimensions without corruption or crash

**Given** the user presses `q`
**When** quit is triggered
**Then** the terminal is restored to its prior state, the alternate buffer is exited, and the shell prompt returns

**Given** an unexpected error occurs internally
**When** the TUI exits
**Then** the terminal is always restored — raw mode is never left active

**Given** the layout is rendered
**When** the terminal is at or above minimum dimensions
**Then** the screen is divided into four regions: stack pane (left), hints pane (right), input line (full width), and mode bar (full width, bottom)

---

### Story 2.2: Stack Pane Display

As a CLI power user,
I want to see the full stack at all times with the most recent value prominent,
So that I always know the current state of my calculation without any mental overhead.

**Acceptance Criteria:**

**Given** values are on the stack
**When** the stack pane renders
**Then** the most recent value (X) is at the bottom of the pane, with older values above it in order

**Given** values are on the stack
**When** the stack pane renders
**Then** each row is labelled numerically from bottom: `1:` (most recent), `2:`, `3:`, … (HP48 convention)

**Given** a value is wider than the available column
**When** the stack pane renders
**Then** the value is truncated with `…` and the most significant digits remain visible

**Given** the stack has more entries than the visible rows
**When** the stack pane renders
**Then** the most recent values fill the visible rows and older entries scroll off the top

**Given** the stack is empty
**When** the stack pane renders
**Then** the pane shows blank rows with no placeholder text or visual noise

**Given** the X register has a value
**When** the stack pane renders
**Then** the X row is visually distinct (bold or accented) compared to other rows

---

### Story 2.3: Mode Bar & Error Line

As a CLI power user,
I want a permanent status strip showing my current mode and clear error feedback when something goes wrong,
So that I always know what state the calculator is in and errors never leave me confused.

**Acceptance Criteria:**

**Given** the TUI is running in normal mode
**When** the mode bar renders
**Then** it shows `[NORMAL]` on the left and the active angle mode and base on the right

**Given** the TUI is in alpha (insert) mode
**When** the mode bar renders
**Then** it shows `[INSERT]` on the left

**Given** HEX base is active
**When** the mode bar renders
**Then** the active hex representation style is also shown (e.g. `0xFF`)

**Given** no error has occurred
**When** the error line renders
**Then** it is completely blank — no text, no separator, no visual noise

**Given** an invalid operation is attempted (e.g. divide by zero, stack underflow)
**When** the error line renders
**Then** it displays a clear description of the error in a distinct color

**Given** an error is showing
**When** the next valid action completes successfully
**Then** the error line clears automatically — no keystroke required to dismiss it

---

### Story 2.4: Number Entry & Normal Mode Operations

As a CLI power user,
I want to type numbers naturally and execute operations with single keystrokes,
So that I can complete the 30-second session without thinking about the interface.

**Acceptance Criteria:**

**Given** the calculator is in normal mode
**When** the user presses any digit key
**Then** alpha (insert) mode is entered automatically and the digit appears in the input buffer

**Given** the calculator is in normal mode
**When** the user presses `i`
**Then** alpha mode is entered and the input buffer is ready for text input

**Given** alpha mode is active with a number in the buffer
**When** the user presses `Enter`
**Then** the value is pushed onto the stack and the calculator returns to normal mode

**Given** alpha mode is active
**When** the user presses `Esc`
**Then** the buffer is cleared and the calculator returns to normal mode with the stack unchanged

**Given** normal mode is active
**When** the user presses `Esc`
**Then** the calculator remains in normal mode (Esc is always safe to press)

**Given** alpha mode is active
**When** the user is typing in the input line
**Then** the typed characters appear in the input line with a visible cursor

**Given** normal mode is active with sufficient stack values
**When** the user presses an operation key (`+`, `-`, `*`, `/`, `^`, `%`, `!`, `s`, `d`, `p`, `r`, `n`)
**Then** the operation executes immediately with no additional keypress required and the stack updates

**Given** normal mode is active with one or more values on the stack
**When** the user presses `Enter` with an empty input buffer
**Then** the top stack value is duplicated (HP convention)

---

### Story 2.5: Clipboard Copy

As a CLI power user,
I want to copy the top stack value to my clipboard with a single keystroke,
So that I can paste results directly into other terminal applications without leaving the calculator.

**Acceptance Criteria:**

**Given** one or more values are on the stack
**When** the user presses `y`
**Then** the top stack value is copied to the system clipboard and the stack is unchanged

**Given** DEC base is active
**When** the value is copied
**Then** it is copied as a plain decimal number

**Given** HEX base is active with a specific style (e.g. `0xFF`)
**When** the value is copied
**Then** it is copied with that representation style prefix (e.g. `0xFF`, `$FF`, `#FF`, or `FFh`)

**Given** the stack is empty
**When** the user presses `y`
**Then** an appropriate error is shown and the clipboard is unchanged

---

## Epic 3: Discoverability — Hints Pane & Chord System

Users can discover and access every MVP operation through the context-sensitive hints pane and chord system, without consulting any external documentation.

### Story 3.1: Static Hints Pane — Categorized Keybinding Grid

As a CLI power user,
I want to see a categorized display of available operations in the hints pane,
So that I can find any operation at a glance without consulting documentation.

**Acceptance Criteria:**

**Given** the TUI is running
**When** the hints pane renders
**Then** it shows operations grouped by category (e.g. ARITHMETIC, STACK OPS, with chord leaders indicated for trig, log, functions, constants, modes, base)

**Given** the hints pane renders
**When** a chord leader is listed
**Then** it is visually indicated as a leader (e.g. `t ›`) so the user knows to press it first

**Given** the hints pane renders
**When** each operation is shown
**Then** the key and a short label are both visible (e.g. `+  add`, `s  swap`)

---

### Story 3.2: Chord System — Leader Keys & Submenus

As a CLI power user,
I want pressing a chord leader to reveal a submenu of related operations in the hints pane,
So that I can discover and execute any grouped operation in two keystrokes with no prior knowledge.

**Acceptance Criteria:**

**Given** normal mode is active
**When** the user presses a chord leader key (`t`, `l`, `f`, `c`, `m`, `x`, `X`)
**Then** the hints pane header changes to show the active category (e.g. `[TRIG]`)
**And** the hints pane body is replaced entirely with that category's operations

**Given** a chord is active and the submenu is showing
**When** the user presses a valid second key
**Then** the corresponding operation executes and the hints pane returns to its normal categorized view

**Given** a chord is active
**When** the user presses `Esc`
**Then** the chord is cancelled, no operation executes, and the hints pane returns to normal

**Given** a chord is active
**When** the user presses a key not in that submenu
**Then** the chord is cancelled gracefully and an appropriate error is shown

**Given** the trig chord is used (`t` then `s`)
**When** the operation executes
**Then** sin is applied to the top stack value using the active angle mode

**Given** any chord category
**When** the submenu is showing
**Then** every operation in that category is visible with its key and label

---

### Story 3.3: Base & Angle Mode Switching

As a CLI power user,
I want to switch the active numeric base and angle mode via chord keys,
So that I can work in hex, binary, or different angle units without leaving the calculator.

**Acceptance Criteria:**

**Given** the base chord is used (`x` then `h`)
**When** HEX base is activated
**Then** all values currently on the stack redisplay in hexadecimal
**And** the mode bar updates to show `HEX`

**Given** any base (DEC, HEX, OCT, BIN) is activated via chord
**When** the stack renders
**Then** all values redisplay in the newly active base

**Given** the angle mode chord is used (`m` then `r`)
**When** RAD mode is activated
**Then** the mode bar updates to show `RAD`
**And** subsequent trig operations use radians

**Given** the hex style chord is used (`X` then a style key)
**When** a hex style is selected (e.g. `0xFF`, `$FF`, `#FF`, `FFh`)
**Then** hex values display with that prefix/suffix style
**And** the mode bar updates to show the active style (only when HEX base is active)

---

### Story 3.4: Context-Sensitive Hints Filtering

As a CLI power user,
I want the hints pane to show only operations that are relevant to my current stack state,
So that the right next action is always obvious and I never see irrelevant or unavailable operations.

**Acceptance Criteria:**

**Given** the stack is empty
**When** the hints pane renders
**Then** it shows operations for pushing values and constants; binary and unary ops are not shown

**Given** exactly one value is on the stack
**When** the hints pane renders
**Then** unary operations (sqrt, sin, ln, etc.) are shown in addition to push/constant operations; binary operations are not shown

**Given** two or more values are on the stack
**When** the hints pane renders
**Then** binary operations (add, subtract, etc.) are shown alongside unary and push operations

**Given** any stack state
**When** the stack changes (value pushed or popped)
**Then** the hints pane updates immediately on the next render to reflect the new state

**Given** any non-empty stack
**When** the hints pane renders
**Then** at least one relevant operation is always visible

---

### Story 3.5: Register Section in Hints Pane

As a CLI power user,
I want the hints pane to show my defined registers and how to recall them,
So that I can see and access my named values without having to remember their names.

**Acceptance Criteria:**

**Given** one or more named registers are defined
**When** the hints pane renders
**Then** a register section appears showing each register name and its current value
**And** the recall command for each register is shown (e.g. `r1 RCL`)

**Given** no registers are defined
**When** the hints pane renders
**Then** no register section appears — there is no empty placeholder, heading, or visual noise

**Given** a register is stored or deleted
**When** the hints pane next renders
**Then** the register section reflects the updated register list immediately

---

### Story 3.6: Responsive Layout — Hints Pane Collapse

As a CLI power user,
I want the layout to adapt gracefully when my terminal is narrow or short,
So that the calculator remains fully usable at any terminal size.

**Acceptance Criteria:**

**Given** the terminal is 80 columns wide or wider
**When** the layout renders
**Then** the full layout is shown: stack pane and hints pane side by side with all categories visible

**Given** the terminal is between 60 and 79 columns wide
**When** the layout renders
**Then** the hints pane is narrowed; labels may be abbreviated but key bindings remain visible

**Given** the terminal is under 60 columns wide
**When** the layout renders
**Then** the hints pane is hidden entirely; stack pane, input line, error line, and mode bar remain fully functional

**Given** the terminal has fewer than 20 rows
**When** the layout renders
**Then** at least 4 stack rows are always preserved; fixed rows (input line, error line, mode bar) are never sacrificed

**Given** the terminal is resized during a session
**When** the new dimensions are applied
**Then** the layout transitions immediately with no corruption, no crash, and no loss of stack data

---

## Epic 4: Power User Features — Registers, Undo & Session

Users can store and recall named values, undo any mistake fearlessly, and trust that calculator state persists reliably across restarts.

### Story 4.1: Named Memory Registers

As a CLI power user,
I want to store values in named registers and recall them at any time,
So that I can preserve intermediate results and build multi-step calculations across time.

**Acceptance Criteria:**

**Given** a value is on the stack
**When** the user enters `myvar STORE`
**Then** the top stack value is stored under the name `myvar` and the stack is unchanged

**Given** a register `myvar` exists
**When** the user enters `myvar RCL`
**Then** the stored value is pushed onto the top of the stack

**Given** a register `myvar` exists
**When** the user enters `myvar DEL` (or equivalent delete command)
**Then** the register is removed and no longer appears in the register list

**Given** one or more registers are defined
**When** the user views the register list
**Then** each register name and its current value are visible

**Given** a register name that does not exist is recalled
**When** the RCL command is issued
**Then** an appropriate error is shown and the stack is unchanged

**Given** a STORE command is issued with an empty stack
**When** the command is executed
**Then** an appropriate error is shown and no register is created or modified

---

### Story 4.2: Full-State Undo & Redo

As a CLI power user,
I want to undo any operation and redo it if I change my mind,
So that I can experiment freely knowing every mistake is reversible.

**Acceptance Criteria:**

**Given** any state-mutating operation has been performed (stack change, register store/delete, mode change)
**When** the user presses `u`
**Then** the complete calculator state is restored to exactly what it was before that operation — stack, registers, and active modes all revert

**Given** an undo has been performed
**When** the user presses `Ctrl-R`
**Then** the undone operation is reapplied and the state advances forward again

**Given** multiple operations have been performed
**When** the user presses `u` repeatedly
**Then** each press steps back through the operation history one at a time

**Given** redo history exists
**When** the user performs any new state-mutating operation
**Then** the redo history is cleared — redo is only available for operations undone in the current chain

**Given** there is nothing left to undo
**When** the user presses `u`
**Then** the stack is unchanged and an appropriate message is shown

**Given** there is nothing to redo
**When** the user presses `Ctrl-R`
**Then** the stack is unchanged and an appropriate message is shown

**Given** undo history has reached the configured depth limit
**When** a new operation is performed
**Then** the oldest undo snapshot is discarded to make room — the most recent history is always preserved

---

### Story 4.3: Session Persistence & SIGTERM Safety

As a CLI power user,
I want my stack and registers to be restored automatically when I reopen rpncalc,
So that I can close the terminal and pick up exactly where I left off without any manual saving.

**Acceptance Criteria:**

**Given** the calculator has values on the stack and named registers defined
**When** the user quits normally with `q`
**Then** the session is saved automatically — no prompt, no confirmation

**Given** a session was saved on a previous run
**When** the calculator is launched again
**Then** the stack and registers are restored to exactly what they were at the end of the previous session

**Given** the calculator is running with session state
**When** the process receives a SIGTERM signal (e.g. terminal closed, system shutdown)
**Then** the session is saved before the process exits and no state is lost

**Given** the session file write is interrupted mid-write
**When** the calculator is next launched
**Then** the previous valid session is loaded — no corrupt or partial state is ever read

**Given** the session file is found to be corrupt on launch
**When** the calculator starts
**Then** it starts with an empty stack and no registers, and an informative message is shown — it never refuses to launch due to a bad session file

**Given** the process is killed with SIGKILL
**When** the calculator is next launched
**Then** the session reflects the last successfully written state — data loss from SIGKILL is a known and accepted boundary, not a bug

**Given** no previous session exists
**When** the calculator is launched
**Then** it starts with an empty stack and no registers — no error is shown

**Given** the user issues a session reset command
**When** the reset is confirmed
**Then** the stack is cleared, all registers are deleted, and the session file is cleared

---

### Story 4.4: Configuration File

As a CLI power user,
I want to configure rpncalc's default behaviour in a config file,
So that my preferred angle mode, base, precision, and undo depth are set automatically on every launch.

**Acceptance Criteria:**

**Given** a `~/.rpncalc/config.toml` file exists with `angle_mode = "rad"`
**When** the calculator launches
**Then** RAD mode is active from the first frame

**Given** a config file sets `base = "hex"`
**When** the calculator launches
**Then** HEX base is active from the first frame

**Given** a config file sets `precision = 10`
**When** float values are displayed
**Then** they are shown to 10 decimal places

**Given** a config file sets `max_undo_history = 50`
**When** undo history grows beyond 50 entries
**Then** the oldest entries are discarded to stay within the limit

**Given** a config file sets `persist_session = false`
**When** the calculator quits and is relaunched
**Then** no session is saved or restored — it always starts fresh

**Given** no config file exists
**When** the calculator launches
**Then** sensible defaults are used (DEG, DEC, precision 15, undo depth 1000, persist_session true) and no error is shown

**Given** the config file contains an invalid value
**When** the calculator launches
**Then** the invalid field is ignored, the default is used, and the calculator starts normally — it never refuses to launch due to a bad config
