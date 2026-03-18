---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
lastStep: 8
status: complete
completedAt: '2026-03-18'
inputDocuments: [prd.md, ux-design-specification.md]
workflowType: 'architecture'
project_name: 'rpncalc'
user_name: 'Boss'
date: '2026-03-18'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**

41 FRs across 10 categories: stack operations (FR1–8), arithmetic and operations (FR9–14), display and modes (FR15–18), discoverability and hints (FR19–22), memory registers (FR23–26), undo and history (FR27–29), session and persistence (FR30–32), clipboard (FR33–34), configuration (FR35–37), and shell integration (FR38–41).

Architecturally, the FRs decompose into three subsystems:
1. **Calculator engine** — stack, registers, operations, modes, undo history
2. **TUI layer** — five widgets (StackPane, InputLine, HintsPane, ErrorLine, ModeBar), event loop, modal input state machine
3. **I/O layer** — session persistence, config loading, clipboard, SIGTERM handling

**Non-Functional Requirements:**

| NFR | Requirement | Architectural implication |
|---|---|---|
| NFR1 | Startup <500ms | Lean initialization; config + session load must be synchronous but fast |
| NFR2 | Keypress-to-render <50ms | Event loop must not block; all ops complete synchronously within one tick |
| NFR3 | Arbitrary precision within 200ms | dashu (IBig/FBig) for computation; f64 intermediate for trig/log is acceptable |
| NFR4 | TUI redraw ≤16ms | ratatui immediate-mode rendering; no per-frame allocations in hot path |
| NFR5 | Atomic session writes | write-to-temp → rename pattern; no partial writes |
| NFR6 | Survive SIGTERM | SIGTERM handler saves session state before exit |
| NFR7/8 | No panics; stack unchanged on error | `Result<>` throughout engine; operations are transactional (pop, compute, push — never partial) |

**Scale & Complexity:**

- Primary domain: Systems/TUI — Rust, ratatui, crossterm, dashu, serde
- Complexity level: **low** (single binary, single user, no network, no database, no multi-tenancy)
- Estimated architectural components: ~8 modules

### Technical Constraints & Dependencies

- **Language:** Rust (single binary, no runtime dependencies)
- **TUI:** ratatui + crossterm (already in scope from PRD)
- **Arithmetic:** dashu (IBig for integers, FBig for arbitrary-precision floats)
- **Serialization:** serde + serde_json for session.json; toml crate for config.toml
- **Clipboard:** arboard or cli-clipboard crate (cross-platform)
- **Config directory:** `~/.rpncalc/` (XDG base dirs via dirs crate)
- **Platform:** Linux primary; standard ANSI/VT100 terminal

### Cross-Cutting Concerns Identified

1. **State ownership** — calculator state (stack + registers + modes) is a single struct; undo history and session persistence both operate on it; TUI reads it for every render
2. **Error propagation** — no panics anywhere; all operations return `Result<>`; engine operations are transactional (never leave partial state)
3. **Undo snapshot strategy** — every state-mutating operation clones and pushes the full calculator state onto the undo stack before executing; redo stack cleared on new operation
4. **Serialization boundary** — calculator state must be fully `Serialize`/`Deserialize`; undo history is in-memory only (not persisted)
5. **HintsPane state machine** — reads calculator state + input mode on every render tick; must be purely functional (no side effects, no owned state)
6. **Terminal resize** — layout must reflow on every `Event::Resize`; ratatui constraint system handles this if no hard-coded dimensions exist

## Starter Template Evaluation

### Primary Technology Domain

Rust systems/TUI binary. No starter template ecosystem equivalent to web frameworks exists — the meaningful decisions are project structure, module organization, and dependency selection.

### Starter Options Considered

| Option | Assessment |
|---|---|
| `cargo new rpncalc --bin` | Minimal single binary scaffold — full control, no cruft |
| ratatui community templates | Useful reference for event loop boilerplate; adds scaffolding we'd partially remove |
| Cargo workspace (multi-crate) | Overkill for this scope — single user tool, low complexity |

### Selected Starter: `cargo new rpncalc --bin`

**Rationale:** Single binary, no inter-crate boundaries needed, existing scaffold in repo can be reused or reset. Full ownership of module structure from day one.

**Initialization:**

```bash
cargo new rpncalc --bin
```

The existing repo scaffold can be retained and reorganized to match the target module structure defined in the architectural decisions step.

### Core Dependencies

| Crate | Purpose | Target version |
|---|---|---|
| `ratatui` | TUI framework — layout, widgets, rendering | ~0.29 |
| `crossterm` | Terminal backend — events, raw mode | ~0.28 |
| `dashu` | Arbitrary precision arithmetic (IBig, FBig) | ~0.4 |
| `serde` + `serde_json` | Session state serialization/deserialization | 1.x |
| `toml` | Config file parsing | ~0.8 |
| `dirs` | XDG config directory resolution (`~/.rpncalc/`) | ~5.0 |
| `arboard` | Cross-platform clipboard access | ~3.x |
| `signal-hook` | SIGTERM handler for clean session save | ~0.3 |

**Note:** Dependency versions to be pinned in `Cargo.toml` at implementation time using `cargo add`. The `dashu` crate (IBig/FBig) is already present in the existing scaffold.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- Module structure (shapes all file organization and import paths)
- Calculator state ownership model (shapes undo, persistence, and rendering)
- Action enum pattern (shapes input testing strategy)

**Important Decisions (Shape Architecture):**
- Event loop model (single-threaded)
- Error handling strategy (Result-based, no panics)

**Deferred Decisions (Post-MVP):**
- Shell completions (FR38–39, Phase 2)
- Unit conversion rules file (FR37, Phase 2)
- Undo tree / branching history (Phase 3)

### Module Structure

Option A — flat modules under `src/`, single binary crate:

```
src/
  main.rs
  engine/
    mod.rs
    stack.rs        # Stack struct, push/pop/swap/dup/drop/rotate
    ops.rs          # apply_op() dispatch, all operation implementations
    undo.rs         # UndoHistory — Vec<CalcState>, push/undo/redo
    value.rs        # CalcValue enum (Integer(IBig), Float(FBig))
    angle.rs        # AngleMode enum + conversion helpers
    base.rs         # Base enum + display helpers
    constants.rs    # π, e, φ as CalcValue
  tui/
    mod.rs
    app.rs          # App struct — owns CalcState + UndoHistory + AppMode
    layout.rs       # ratatui layout constraints, terminal size handling
    widgets/
      stack_pane.rs
      input_line.rs
      hints_pane.rs
      error_line.rs
      mode_bar.rs
  input/
    mod.rs
    action.rs       # Action enum — every user-triggerable event
    handler.rs      # handle_key(AppMode, KeyEvent) -> Action  [pure fn]
    mode.rs         # AppMode enum (Normal, Alpha(String), Chord(ChordCategory))
    parser.rs       # parse_value(str) -> Result<CalcValue, String>
    commands.rs     # parse_command(str) -> Result<Action, String>
  config/
    mod.rs
    config.rs       # Config struct + config.toml loading
    session.rs      # SessionState serde + atomic write/read
```

### Calculator State Ownership

Single `CalcState` struct owns all mutable calculator state:

```rust
pub struct CalcState {
    pub stack: Vec<CalcValue>,
    pub registers: HashMap<String, CalcValue>,
    pub angle_mode: AngleMode,
    pub base: Base,
    pub hex_style: HexStyle,
}
```

`CalcState` derives `Clone`, `Serialize`, `Deserialize`. Undo history is `Vec<CalcState>` — snapshot before every state-mutating operation. Session persistence serializes `CalcState` to `~/.rpncalc/session.json`. The TUI renders from `&CalcState` — read-only borrow per frame.

### Event Loop Architecture

Single-threaded blocking event loop — no async, no threads:

```rust
loop {
    terminal.draw(|f| render(f, &app))?;
    match crossterm::event::read()? {
        Event::Key(key) => {
            let action = input::handler::handle_key(&app.mode, key);
            app.apply(action)?;
        }
        Event::Resize(_, _) => { /* layout reflows automatically */ }
        _ => {}
    }
    if app.should_quit { break; }
}
```

### Error Handling Strategy

All engine operations return `Result<(), CalcError>` or `Result<CalcValue, CalcError>`. The `App::apply(action)` method catches errors and sets `app.error_message: Option<String>`. The `ErrorLine` widget renders `app.error_message`. Error message clears on next successful action. No `unwrap()` in non-test code except truly unreachable paths (must be documented with `// SAFETY:` comment).

### Testing Strategy

**Engine unit tests** — pure functions, no TUI dependency. Every operation in `ops.rs` has test coverage. Every `parse_value` variant tested.

**Keybinding unit tests** — `handle_key()` is a pure function:
```rust
#[test]
fn test_chord_trig_sin() {
    let action = handle_key(&AppMode::Chord(ChordCategory::Trig), key('s'));
    assert_eq!(action, Action::Op("sin"));
}
```
Every binding in the Normal mode table, every chord leader, and every chord second key has a corresponding test. This is the primary regression safety net for the interaction model.

**Session serialization tests** — round-trip: `CalcState` → JSON → `CalcState` equality.

**TUI widgets** — not unit-tested (ratatui rendering requires a real terminal buffer). Manual testing for layout + resize behaviour.

### Decision Impact Analysis

**Implementation sequence implications:**
1. `CalcState` + `CalcValue` structs must be defined first — everything depends on them
2. `Action` enum defined before `handler.rs` or `ops.rs`
3. Engine ops implemented and tested before TUI is wired up
4. `handle_key()` implemented and tested before integrating into the event loop
5. Session serialization tested before SIGTERM handler is written

**Cross-component dependencies:**
- `HintsPane` needs read access to `CalcState` + `AppMode` to determine display state
- `ErrorLine` reads `app.error_message: Option<String>` — set by `App::apply()`, cleared on next success
- Undo history snapshots `CalcState` — requires `Clone`; session persistence requires `Serialize`/`Deserialize` — both on the same struct

## Implementation Patterns & Consistency Rules

### Critical Conflict Points Identified

6 areas where AI agents could make inconsistent choices without explicit rules.

### Naming Patterns

**Stack terminology:**
- Top of stack = "X register" in user-facing display; `stack.last()` in code
- Stack access: `stack.last()` = X, `stack[stack.len()-2]` = Y — never use index 0 for "top"
- Register map key type: `String` (owned), never `&str` in `HashMap`

**Error type:**
- Single `CalcError` enum in `src/engine/error.rs`, re-exported as `engine::CalcError`
- Variants use descriptive names: `CalcError::StackUnderflow`, `CalcError::DivisionByZero`, `CalcError::DomainError(String)`
- No per-module error types — all engine errors are `CalcError`

**Code naming (Rust conventions enforced by clippy/rustfmt):**
- Functions/variables/modules: `snake_case`
- Types/enums/traits: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Enum variants: `PascalCase`

### Structure Patterns

**Test location:**
- Unit tests: co-located `#[cfg(test)]` module at the bottom of every source file
- Integration tests: `tests/` directory for end-to-end scenarios (e.g., session round-trip, full key sequence)
- No mixing — never put integration-style tests in `#[cfg(test)]` blocks

**Module visibility:**
- Each module exposes only what is needed via `pub` — prefer `pub(crate)` for internal cross-module APIs
- `pub use` re-exports at module root (`engine/mod.rs`) for the primary public types

### Format Patterns

**Session JSON format:**
- Field names: `snake_case` (serde default for Rust structs)
- Stack: JSON array, bottom of stack at index 0
- Registers: JSON object with string keys
- Modes: string literals matching config.toml values (`"deg"`, `"hex"`, etc.)

**Config TOML format:**
- All keys `snake_case`
- Matches the schema defined in the PRD exactly — no ad-hoc additions

### Communication Patterns

**Operation dispatch — Action enum is the sole dispatch mechanism:**
```rust
// CORRECT: all user actions flow through Action enum
app.apply(Action::Op(Op::Sin))
app.apply(Action::Push(CalcValue::Integer(42.into())))
app.apply(Action::Undo)

// FORBIDDEN: string dispatch in new code
apply_op(engine, "sin")  // only in legacy code pending refactor
```

**State mutation pattern — snapshot before execution:**
```rust
// In App::apply():
self.undo_history.push(self.state.clone());  // snapshot first
match self.state.apply(action) {
    Ok(_) => { self.error = None; }
    Err(e) => {
        self.undo_history.pop();  // discard snapshot on failure
        self.error = Some(e.to_string());
    }
}
```
Stack state is NEVER partially modified — operations are atomic.

### Process Patterns

**Error handling — `?` operator throughout engine:**
```rust
// CORRECT
fn do_div(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    if b.is_zero() { return Err(CalcError::DivisionByZero); }
    Ok(a / b)
}

// FORBIDDEN in engine code
let val = something.unwrap();
```

**No `unwrap()` policy:**
- Engine code: `?` operator only
- TUI render code: `unwrap_or_default()` or `unwrap_or("")` at worst — panicking in render loop is unacceptable
- Tests: `unwrap()` is acceptable

### Enforcement Guidelines

**All AI agents MUST:**
- Run `cargo clippy -- -D warnings` before considering any task complete
- Run `cargo fmt` — no manual formatting
- Add `#[cfg(test)]` unit tests for every new public function in `engine/` and `input/`
- Never introduce new `unwrap()` calls in non-test code without a `// SAFETY:` comment explaining why it's unreachable

**Anti-patterns to reject in code review:**
- String-based op dispatch (`apply_op(engine, "sin")`) in new code
- `HashMap::get().unwrap()` without bounds checking
- Direct stack index access without length check
- Mutable borrow of `CalcState` inside a render function

## Project Structure & Boundaries

### Complete Project Directory Structure

```
rpncalc/
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── README.md
├── src/
│   ├── main.rs                    # binary entry point, event loop, terminal setup/teardown
│   ├── engine/
│   │   ├── mod.rs                 # pub use re-exports: CalcState, CalcValue, CalcError
│   │   ├── error.rs               # CalcError enum  (FR7, FR8, NFR7, NFR8)
│   │   ├── value.rs               # CalcValue enum: Integer(IBig), Float(FBig)  (FR1)
│   │   ├── stack.rs               # Stack ops: push/pop/swap/dup/drop/rotate/clear  (FR1-8)
│   │   ├── ops.rs                 # All operation impls: arith, trig, log, fn, bitwise  (FR9-13)
│   │   ├── constants.rs           # π, e, φ as CalcValue  (FR14)
│   │   ├── angle.rs               # AngleMode enum + to/from radians  (FR16)
│   │   ├── base.rs                # Base + HexStyle enums + display  (FR15, FR17)
│   │   ├── registers.rs           # Register map: store/recall/delete/list  (FR23-26)
│   │   └── undo.rs                # UndoHistory: Vec<CalcState>, push/undo/redo  (FR27-29)
│   ├── input/
│   │   ├── mod.rs
│   │   ├── action.rs              # Action enum — all dispatchable events
│   │   ├── mode.rs                # AppMode enum: Normal, Alpha(String), Chord(ChordCategory)
│   │   ├── handler.rs             # handle_key(AppMode, KeyEvent) -> Action  [pure fn, tested]
│   │   ├── parser.rs              # parse_value(str) -> Result<CalcValue, CalcError>  (FR1)
│   │   └── commands.rs            # parse_command(str) -> Result<Action, CalcError>  (FR40)
│   ├── tui/
│   │   ├── mod.rs
│   │   ├── app.rs                 # App struct: owns CalcState + UndoHistory + AppMode + error
│   │   ├── layout.rs              # ratatui layout constraints, terminal size handling
│   │   └── widgets/
│   │       ├── mod.rs
│   │       ├── stack_pane.rs      # StackPane widget  (FR18)
│   │       ├── input_line.rs      # InputLine widget
│   │       ├── hints_pane.rs      # HintsPane widget + state machine  (FR19-22, FR41)
│   │       ├── error_line.rs      # ErrorLine widget  (NFR7)
│   │       └── mode_bar.rs        # ModeBar widget  (FR22)
│   └── config/
│       ├── mod.rs
│       ├── config.rs              # Config struct + config.toml loading  (FR35-36)
│       └── session.rs             # SessionState serde + atomic write/read  (FR30-32, NFR5-6)
├── tests/
│   └── integration/
│       ├── session_roundtrip.rs   # serialize → deserialize CalcState equality
│       └── keybinding_sequences.rs # multi-step key sequence integration tests
└── ~/.rpncalc/                    # runtime data (not in repo)
    ├── config.toml
    ├── session.json
    └── history.toml               # (Phase 2)
```

### Architectural Boundaries

**Engine boundary:**
- `engine/` is a pure computation module — no TUI imports, no I/O, no crossterm
- All engine functions take and return owned or borrowed values — no global state
- The only dependency engine has on the rest of the codebase: none (it is the bottom of the dependency graph)

**Input boundary:**
- `input/` depends on `engine/` (for `CalcValue`, `CalcError`, `Action` targets) but not on `tui/`
- `handler.rs` is a pure function — takes `&AppMode` and `KeyEvent`, returns `Action`
- `parser.rs` and `commands.rs` return `Result<_, CalcError>` — no UI concerns

**TUI boundary:**
- `tui/` depends on both `engine/` and `input/` but owns no computation logic
- Widgets receive read-only references: `render_stack_pane(f, area, &state)` — no mutation in render
- `app.rs` is the only place that mutates `CalcState` — all mutations go through `App::apply(action)`

**Config boundary:**
- `config/` depends on `engine/` (to serialize `CalcState`) but not on `tui/` or `input/`
- Session writes are always atomic (write temp → rename)
- SIGTERM handler in `main.rs` calls `session::save(&state)` directly

### Requirements to Structure Mapping

| FR group | Primary file(s) |
|---|---|
| Stack ops (FR1–8) | `engine/stack.rs`, `engine/value.rs` |
| Operations (FR9–14) | `engine/ops.rs`, `engine/constants.rs` |
| Display/modes (FR15–18) | `engine/base.rs`, `engine/angle.rs`, `tui/widgets/stack_pane.rs` |
| Hints/discoverability (FR19–22) | `tui/widgets/hints_pane.rs` |
| Registers (FR23–26) | `engine/registers.rs` |
| Undo/redo (FR27–29) | `engine/undo.rs` |
| Session/persistence (FR30–32) | `config/session.rs` |
| Clipboard (FR33–34) | `tui/app.rs` (arboard call site) |
| Config (FR35–36) | `config/config.rs` |
| Register commands (FR40–41) | `input/commands.rs`, `tui/widgets/hints_pane.rs` |

### Integration Points

**Internal data flow:**
```
KeyEvent
  → input::handler::handle_key(&mode, event) → Action
  → tui::app::App::apply(action)
      → engine snapshot → engine mutation → Result
      → error_message updated if Err
  → terminal.draw(|f| tui::layout::render(f, &app))
      → widgets read &app.state, &app.mode, &app.error
```

**External integrations:**
- `~/.rpncalc/config.toml` — read once at startup via `config::Config::load()`
- `~/.rpncalc/session.json` — read at startup, written atomically on exit/SIGTERM
- System clipboard — `arboard::Clipboard` called in `App::apply(Action::Yank)`
- Terminal — `crossterm` raw mode + `ratatui::Terminal` wrap/restore in `main.rs`

### Development Workflow

**Build:** `cargo build --release` → single binary at `target/release/rpncalc`
**Test:** `cargo test` — unit tests (co-located) + integration tests (`tests/`)
**Lint:** `cargo clippy -- -D warnings`
**Format:** `cargo fmt`
**Run:** `cargo run` during development

## Architecture Validation Results

### Coherence Validation ✅

**Decision compatibility:** All technology choices are compatible — ratatui + crossterm is the standard pairing, dashu is independent of the TUI layer, serde supports both JSON (session) and TOML (config) targets without conflict.

**Pattern consistency:** `Action` enum as sole dispatch mechanism is consistent with the pure `handle_key()` function pattern. No string dispatch in new code. Patterns are enforceable via clippy.

**Structure alignment:** Module boundaries are clean and acyclic. `engine/` has zero upward dependencies. `input/` depends only on `engine/`. `tui/` depends on both. `config/` depends on `engine/` only. `main.rs` depends on all four — appropriate for a binary entry point.

### Requirements Coverage Validation ✅

**Functional requirements:** All 41 FRs mapped to specific files in the project structure. No FR without a home.

**Non-functional requirements:**

| NFR | Architecture address |
|---|---|
| NFR1 startup <500ms | Lean synchronous init: config load → session read → first frame |
| NFR2 keypress <50ms | Blocking sync event loop; all ops complete in single tick |
| NFR3 precision 200ms | dashu IBig/FBig; f64 intermediate acceptable for trig/log |
| NFR4 redraws ≤16ms | ratatui immediate-mode; no per-frame heap allocations in hot path |
| NFR5 atomic writes | write-temp → rename in `session.rs` |
| NFR6 SIGTERM survival | signal-hook handler in `main.rs` calls `session::save()` |
| NFR7/8 no panics | `Result<>` throughout engine; `App::apply()` catches all errors |

### Implementation Readiness Validation ✅

**Decision completeness:** All critical decisions documented — module structure, state ownership, event loop model, error handling, testing strategy, keybinding testability.

**Structure completeness:** Every source file named, purpose documented, FR mapping provided.

**Pattern completeness:** Naming, structure, dispatch, mutation order, error handling, and test placement all specified with examples.

### Gap Analysis

| Gap | Priority | Resolution |
|---|---|---|
| `CalcState` struct fields not yet code-defined | Important | Define in first implementation story — fields known |
| `Action` enum variants not fully enumerated | Important | Define during story implementation with keybinding table as spec |
| HintsPane state machine logic not detailed | Nice-to-have | Warrants its own story with detailed acceptance criteria |
| Clipboard crate confirmation (arboard) | Minor | Confirm at implementation time; arboard is standard |

### Architecture Completeness Checklist

- [x] Project context analyzed and scale assessed
- [x] Technical constraints and cross-cutting concerns identified
- [x] Starter/project structure approach defined
- [x] Core dependencies selected
- [x] Module structure defined
- [x] Calculator state ownership model defined
- [x] Event loop architecture defined
- [x] Error handling strategy defined
- [x] Testing strategy defined (including keybinding testability)
- [x] Implementation patterns and consistency rules documented
- [x] Complete project directory structure with FR mapping
- [x] Architectural boundaries defined
- [x] Data flow documented

### Architecture Readiness Assessment

**Overall status: READY FOR IMPLEMENTATION**

**Confidence level: High**

**Key strengths:**
- Clean acyclic module dependencies — engine is fully isolated and testable
- Action enum pattern makes the entire interaction model unit-testable
- Single `CalcState` struct satisfies both undo and persistence requirements without duplication
- All 41 FRs and 11 NFRs have explicit architectural support

**Note for first implementation story:** The existing scaffold uses a different structure (string-based op dispatch, no `registers.rs`, no `undo.rs`). Story 1 should reorganize the scaffold to match the target structure before building new features.

**AI Agent Implementation Guidelines:**
- Follow module structure exactly — do not create new top-level modules without architectural review
- All operations route through `Action` enum — no string dispatch in new code
- `CalcState` is the only mutable state — never introduce secondary state stores
- Snapshot before execute — undo correctness depends on this order
- `cargo clippy -- -D warnings` must pass before any task is considered complete
