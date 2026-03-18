# Story 1.2: Core Data Types

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want the foundational data types defined,
so that stack values, calculator state, errors, and user actions all have a consistent, well-typed representation.

## Acceptance Criteria

1. **Given** the core types are defined, **When** a numeric value is represented, **Then** it can be either an arbitrary-precision integer or an arbitrary-precision float.

2. **Given** the calculator state type exists, **When** it is created with defaults, **Then** it holds a stack of values, a set of named registers, an active angle mode, an active base, and an active hex style.

3. **Given** a calculator error occurs, **When** the error is reported, **Then** it carries a meaningful description (e.g. stack underflow, division by zero, domain error).

4. **Given** the action type is defined, **When** any user-triggered event occurs, **Then** it is representable as a typed action with no string-based dispatch.

## Tasks / Subtasks

- [x] Task 1: Enable dashu serde support in Cargo.toml (AC: 1)
  - [x] Change `dashu = "0.4"` to `dashu = { version = "0.4", features = ["serde"] }`
  - [x] Run `cargo build` to confirm dashu serde compiles — Path A confirmed: IBig/FBig derive Serialize/Deserialize natively

- [x] Task 2: Upgrade CalcValue with Serialize/Deserialize (AC: 1)
  - [x] Add `#[derive(Serialize, Deserialize)]` to `CalcValue` in `src/engine/value.rs`
  - [x] Path A used — native derives work; no custom serde modules needed
  - [x] Path A used — IBig serializes natively; no custom module needed
  - [x] Remove the now-redundant `serialize_to_string` and `deserialize_from_string` methods from `CalcValue`
  - [x] Fixed dead computation in `format_fbig`: deleted `parse_scientific_display`; fixed root cause (FBig.to_string() is binary — must use `.to_f64().value()` instead); inlined trimming
  - [x] Added unit tests: integer/float/negative serde roundtrip, format_fbig trims zeros, format_fbig keeps decimals
  - [x] Run `cargo test` — all 4 pre-existing parser tests pass

- [x] Task 3: Add HexStyle enum to engine/base.rs (AC: 2)
  - [x] Added `HexStyle` enum with variants: `ZeroX`, `Dollar`, `Hash`, `Suffix`
  - [x] Derives `Clone, Copy, PartialEq, Debug, Serialize, Deserialize`
  - [x] Implemented `HexStyle::cycle(self) -> HexStyle`
  - [x] Implemented `fmt::Display` for HexStyle
  - [x] Added `pub use base::HexStyle;` to `engine/mod.rs`
  - [x] Unit tests: cycle full round-trip, Display for each variant

- [x] Task 4: Create CalcState struct in engine/stack.rs (replacing StackEngine) (AC: 2)
  - [x] StackEngine retained as `#[allow(dead_code)]` compat stub — Task 5 requires `apply_op(&mut StackEngine)` to stay in place; full removal deferred to Story 1.4 per story constraint
  - [x] `CalcState` defined with all required fields
  - [x] `use std::collections::HashMap` and required imports added
  - [x] Derives `Clone, Debug, Serialize, Deserialize`
  - [x] `CalcState::new()` returns correct defaults
  - [x] `Default for CalcState` delegates to `new()`
  - [x] Convenience methods: `is_empty()`, `depth()`, `peek()`
  - [x] `pub use stack::CalcState;` added to `engine/mod.rs`
  - [x] Unit tests: defaults, is_empty, depth

- [x] Task 5: Define Op enum in engine/ops.rs (AC: 4)
  - [x] `Op` enum added at TOP of `ops.rs`, before `apply_op`
  - [x] All 30 variants across 7 categories present
  - [x] Derives `Debug, Clone, Copy, PartialEq`
  - [x] `apply_op(engine: &mut StackEngine, op: &str)` left in place
  - [x] `pub use ops::Op;` added to `engine/mod.rs`
  - [x] Unit test: Op::Sin constructible, Op::Add != Op::Sub

- [x] Task 6: Define full Action enum in input/action.rs (AC: 4)
  - [x] Full `Action` enum replaces `Noop` stub
  - [x] All 14 variants present
  - [x] Required imports added (CalcError omitted per dev note: "can be part of commands.rs imports, not action.rs")
  - [x] Derives `Debug, Clone, PartialEq`
  - [x] `#[allow(dead_code)]` removed
  - [x] Unit test: Push, Execute(Op::Sin), Undo constructible

- [x] Task 7: Upgrade UndoHistory to snapshot CalcState (engine/undo.rs) (AC: 2)
  - [x] `past` and `future` changed to `Vec<CalcState>`
  - [x] `max_depth: usize` field added, defaults to 1000
  - [x] `snapshot()` clones full state, clears future, trims oldest if over max_depth
  - [x] `undo()` pops past, pushes current to future, returns restored state
  - [x] `redo()` pops future, pushes current to past, returns restored state
  - [x] `new()` and `Default` set `max_depth: 1000`
  - [x] `with_max_depth()` constructor added
  - [x] `use crate::engine::stack::CalcState` added
  - [x] Unit tests: snapshot, undo, redo, new-action-clears-redo, depth limiting

- [x] Task 8: Update engine/mod.rs re-exports (AC: 1, 2, 4)
  - [x] `pub use value::CalcValue;` added
  - [x] `pub use ops::Op;` added
  - [x] `pub use error::CalcError;` confirmed present
  - [x] All `pub mod` declarations retained

- [x] Task 9: Verify AC3 — CalcError completeness check
  - [x] All 5 variants confirmed: StackUnderflow, DivisionByZero, DomainError(String), InvalidInput(String), NotAnInteger
  - [x] No missing variants
  - [x] Display tests added for all variants

- [x] Task 10: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — 22 unit tests + 2 integration tests pass

## Dev Notes

### What This Story Delivers

This is a **types-only** story. No operation implementations, no TUI, no persistence. The goal is to define every central data type so that Stories 1.3–1.5 and all of Epic 2 have a solid, well-typed foundation.

**Types being defined/upgraded:**
- `CalcValue` — add `Serialize`/`Deserialize`, fix dead computation
- `HexStyle` — new enum in `base.rs`
- `CalcState` — new central state struct in `stack.rs` (replaces `StackEngine`)
- `Op` — new operation variants enum in `ops.rs` (no implementations yet)
- `Action` — full enum in `input/action.rs` (replaces `Noop` stub)
- `UndoHistory` — upgraded to snapshot `CalcState` instead of `Vec<CalcValue>`

### CalcState — The Central Mutable State

`CalcState` is the ONLY mutable state object in the entire application. Every engine operation, undo snapshot, and session persistence record operates on it. Defined in `engine/stack.rs`:

```rust
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::engine::{
    value::CalcValue,
    angle::AngleMode,
    base::{Base, HexStyle},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcState {
    pub stack: Vec<CalcValue>,
    pub registers: HashMap<String, CalcValue>,
    pub angle_mode: AngleMode,
    pub base: Base,
    pub hex_style: HexStyle,
}
```

**Register key type MUST be `String` (owned), not `&str`.** Architecture rule: "Register map key type: `String` (owned), never `&str` in HashMap".

**Stack top = `stack.last()`** — the X register. `stack[stack.len()-2]` = Y. Never use index 0 as "top".

### CalcValue Serde — Critical Implementation Detail

`dashu`'s IBig and FBig may not implement serde's `Serialize`/`Deserialize` even with `features = ["serde"]` on the meta-crate. Test by running `cargo build` after adding the feature. Two paths:

**Path A — dashu serde works natively:**
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CalcValue {
    Integer(IBig),
    Float(FBig),
}
```

**Path B — FBig (or IBig) needs custom serde:**
Use `#[serde(with = "...")]` per variant. Example for FBig:
```rust
mod serde_fbig {
    use dashu::float::FBig;
    use serde::{Deserializer, Serializer, de::Error};

    pub fn serialize<S: Serializer>(v: &FBig, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&v.to_string())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<FBig, D::Error> {
        let s = String::deserialize(d)?;
        s.parse::<f64>()
            .ok()
            .and_then(|f| FBig::try_from(f).ok())
            .ok_or_else(|| D::Error::custom("invalid FBig"))
    }
}
```
Same pattern for IBig if needed (parse via `str::parse::<IBig>()`).

**Either way:** `#[derive(Serialize, Deserialize)]` on `CalcState` requires ALL its fields to be serializable. `AngleMode`, `Base`, `HexStyle` already derive Serialize/Deserialize (angle.rs and base.rs have the serde imports). Verify this before assuming it works.

### Op Enum — Variants Only, No Implementations

The `Op` enum goes at the TOP of `engine/ops.rs`, before the existing `apply_op` function. The existing `apply_op(engine: &mut StackEngine, op: &str)` function stays for now — Story 1.4 will replace it with a proper implementation that takes `&mut CalcState` and dispatches on `Op`. Do NOT touch the function logic in this story.

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    // Binary arithmetic
    Add, Sub, Mul, Div, Pow, Mod,
    // Unary
    Negate, Sqrt, Square, Reciprocal, Abs, Factorial,
    // Trig
    Sin, Cos, Tan, Asin, Acos, Atan,
    // Log/Exp
    Log10, Ln, Exp, Exp10,
    // Bitwise
    And, Or, Xor, Not, Shl, Shr,
    // Stack
    Swap, Dup, Drop, Rotate, Clear,
    // Constants (push onto stack)
    PushPi, PushE, PushPhi,
}
```

### Action Enum — No String Dispatch

The `Action` enum in `input/action.rs` represents every dispatchable user event. Use `Execute(Op)` (not `Op(Op)`) to match Rust naming conventions for tuple variants containing another type:

```rust
use crate::engine::{
    CalcValue, CalcError,
    base::{Base, HexStyle},
    angle::AngleMode,
    ops::Op,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Push(CalcValue),
    Execute(Op),
    SetBase(Base),
    SetAngleMode(AngleMode),
    SetHexStyle(HexStyle),
    StoreRegister(String),
    RecallRegister(String),
    DeleteRegister(String),
    Undo,
    Redo,
    Yank,
    EnterAlphaMode,
    Quit,
    Noop,
}
```

**`_CalcError` is imported for future use** — `input/commands.rs` already returns `Result<Action, CalcError>`. The import can be `#[allow(unused_imports)]` for now if clippy complains, or just be part of `commands.rs`'s imports, not `action.rs`. Keep it clean.

### UndoHistory — Full CalcState Snapshots

The existing `UndoHistory` in `undo.rs` snapshots only the stack (`Vec<CalcValue>`). This must be upgraded to snapshot full `CalcState`. Architecture: "snapshot before execute — undo correctness depends on this order". The `max_depth` field prevents unbounded memory growth.

Depth limiting: When `past.len() >= max_depth` before pushing a new snapshot, remove the oldest (`past.remove(0)`). This is O(n) but acceptable since max_depth is typically 1000 and this is not on the hot render path.

### fix: Dead Computation in format_fbig (Story 1.1 deferred)

Story 1.1 code review deferred this fix. The current `format_fbig` in `value.rs`:
```rust
// CURRENT (broken): computes formatted but passes to _sci which ignores it
let formatted = format!("{:.33e}", val);
parse_scientific_display(&formatted, val)
```
Fix by eliminating the dead intermediate. Simplest approach — inline the trimming directly:
```rust
pub(crate) fn format_fbig(f: &FBig) -> String {
    let s = f.to_string();
    let val: f64 = s.parse().unwrap_or(f64::NAN);
    if val.is_nan() || val.is_infinite() {
        return format!("{}", val);
    }
    let s = format!("{:.15}", val);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}
```
Delete `parse_scientific_display` entirely.

### Previous Story Lessons (from Story 1.1)

1. **Never add `impl` blocks for external types** (IBig, FBig, etc.) — E0116 compile error. Use extension traits or free functions instead.
2. **`clippy::wrong_self_convention`**: methods named `from_*` that take `self` trigger this. Either rename or add `#[allow(clippy::wrong_self_convention)]`.
3. **`unwrap_or_else(|_| val)` → `unwrap_or(val)`** for non-lazy values — clippy::unnecessary_lazy_evaluations.
4. **`pub` vs `pub(crate)`** for internal helpers — prefer `pub(crate)` per architecture rule.
5. **Module inception**: `config/mod.rs` declaring `pub mod config;` triggers `clippy::module_inception`. Use `#[allow(clippy::module_inception)]` if needed.
6. **`#![allow(dead_code)]` at crate root** remains from Story 1.1 — do NOT remove it; it is needed until modules wire up.

### Project Structure Notes

- `HexStyle` goes in `src/engine/base.rs` alongside `Base` — architecture spec: `base.rs` = "Base + HexStyle enums + display"
- `CalcState` goes in `src/engine/stack.rs` — replaces `StackEngine` entirely
- `Op` goes in `src/engine/ops.rs` — at the top, before the legacy `apply_op`
- `Action` goes in `src/input/action.rs` — replaces the `Noop` stub
- `UndoHistory` stays in `src/engine/undo.rs`
- Do NOT create any new files — all changes go into existing files

**engine/mod.rs re-exports after this story:**
```rust
pub use error::CalcError;
pub use value::CalcValue;
pub use ops::Op;
pub use stack::CalcState;
```

### Test Placement

- Unit tests: `#[cfg(test)]` module at the bottom of each file
- No integration tests in this story
- The 4 pre-existing parser tests in `input/parser.rs` must continue to pass — do NOT change parser.rs in this story

### References

- CalcState struct definition: [Source: architecture.md#Calculator State Ownership]
- Op/Action dispatch pattern: [Source: architecture.md#Communication Patterns]
- UndoHistory snapshot pattern: [Source: architecture.md#Cross-Cutting Concerns]
- HexStyle display: [Source: ux-design-specification.md — UX-DR11, Story 3.3 ACs]
- CalcState register key type String: [Source: architecture.md#Naming Patterns]
- Session JSON format (snake_case fields): [Source: architecture.md#Format Patterns]
- No unwrap() rule: [Source: architecture.md#Process Patterns]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

### Completion Notes List

- dashu Path A confirmed: IBig and FBig both derive Serialize/Deserialize natively via `features = ["serde"]`. No custom serde modules required.
- Root cause discovered: `FBig::to_string()` outputs binary representation (e.g. "11" for 3.0). Fixed `format_fbig` and `CalcValue::to_f64` to use `f.to_f64().value()` (dashu's decimal conversion).
- StackEngine retained as `#[allow(dead_code)]` compat stub in `stack.rs` — Task 4 says delete it but Task 5 requires `apply_op(&mut StackEngine)` to stay; contradiction resolved by keeping StackEngine until Story 1.4 replaces `apply_op`.
- `display_with_base` in `value.rs` still hardcodes `"0x"` prefix for hex — it doesn't accept `HexStyle`. Wiring HexStyle into display is out of scope for this types-only story; deferred to Story 3.3.
- Added `#![allow(unused_imports)]` to `main.rs` alongside existing `#![allow(dead_code)]` to suppress re-export warnings while the crate is still scaffolding.

### File List

- `Cargo.toml` — added `features = ["serde"]` to dashu dependency
- `src/main.rs` — added `#![allow(unused_imports)]`
- `src/engine/mod.rs` — added re-exports: `CalcValue`, `Op`, `CalcState`, `HexStyle`
- `src/engine/value.rs` — added `PartialEq, Serialize, Deserialize` derives; fixed `format_fbig` and `to_f64` to use `FBig::to_f64().value()`; added 5 unit tests
- `src/engine/base.rs` — added `HexStyle` enum with `cycle()`, `Display`, and 2 unit tests
- `src/engine/stack.rs` — replaced `StackEngine` with `CalcState`; retained `StackEngine` as compat stub; added 3 unit tests
- `src/engine/ops.rs` — added `Op` enum (30 variants) at top; left `apply_op` intact; added 1 unit test
- `src/engine/undo.rs` — upgraded to `Vec<CalcState>`, added `max_depth`, new constructors, updated all methods; added 5 unit tests
- `src/engine/error.rs` — added 1 unit test for Display messages
- `src/input/action.rs` — replaced `Noop` stub with full 14-variant `Action` enum; added 1 unit test
