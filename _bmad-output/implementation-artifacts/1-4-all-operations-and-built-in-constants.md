# Story 1.4: All Operations & Built-in Constants

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a CLI power user,
I want to apply arithmetic, trig, log/exp, utility, and bitwise operations, and push built-in constants,
So that I can perform every calculation the calculator offers.

## Acceptance Criteria

1. **Given** two values on the stack, **When** a binary arithmetic operation (add, subtract, multiply, divide, power, modulo) is applied, **Then** the result replaces the top two values and stack depth decreases by one.

2. **Given** a division by zero is attempted, **When** divide is applied, **Then** the stack is unchanged and `CalcError::DivisionByZero` is returned.

3. **Given** one value on the stack with DEG angle mode active, **When** sin, cos, or tan is applied, **Then** the result treats the input as degrees.

4. **Given** one value on the stack, **When** an inverse trig operation (asin, acos, atan) is applied, **Then** the result is expressed in the active angle unit (degrees, radians, or gradians).

5. **Given** one value on the stack, **When** ln, log10, exp, or exp10 is applied, **Then** the mathematically correct result is pushed.

6. **Given** one value on the stack, **When** sqrt, square, reciprocal, absolute value, negate, or factorial is applied, **Then** the mathematically correct result is pushed.

7. **Given** an operation with no valid result (e.g. sqrt of a negative, ln of zero or negative, asin/acos of out-of-range value), **When** the operation is attempted, **Then** the stack is completely unchanged and `CalcError::DomainError(_)` is returned.

8. **Given** an integer on the stack (two integers for binary bitwise), **When** a bitwise operation (AND, OR, XOR, NOT, shift left, shift right) is applied, **Then** the correct bitwise result is pushed.

9. **Given** a float is passed to a bitwise operation, **When** the operation is attempted, **Then** the stack is unchanged and `CalcError::NotAnInteger` is returned.

10. **When** a built-in constant (π, e, φ) is pushed, **Then** the constant's value appears on top of the stack as a `CalcValue::Float`.

11. **Given** any operation that fails (any error), **When** the error is returned, **Then** the stack state is completely unchanged — no partial mutation.

## Tasks / Subtasks

- [x] Task 1: Rewrite `src/engine/ops.rs` with new dispatch and atomic helpers (AC: 1–11)
  - [x] Add imports: `use crate::engine::stack::CalcState;`, `use crate::engine::error::CalcError;`, `use crate::engine::constants;`
  - [x] Remove old imports: `use crate::engine::stack::StackEngine;` (StackEngine is being deleted)
  - [x] Add new `pub fn apply_op(state: &mut CalcState, op: Op) -> Result<(), CalcError>` matching all 33 Op variants
  - [x] Replace `binary_op` helper to use `&mut CalcState` with peek-then-mutate atomicity pattern (see Dev Notes)
  - [x] Replace `unary_op` helper to use `&mut CalcState` with peek-then-mutate atomicity pattern (see Dev Notes)
  - [x] Add `fn do_negate(v: CalcValue) -> Result<CalcValue, CalcError>` (was missing from original)
  - [x] Convert all `do_*` functions from `Result<CalcValue, String>` to `Result<CalcValue, CalcError>`
  - [x] Fix `do_abs` float branch — replace `x.to_string().parse::<f64>()` with `x.to_f64().value()` (FBig binary string bug)
  - [x] Remove old `apply_op(&mut StackEngine, op: &str)` function entirely
  - [x] Remove `fn op_symbol` helper (was only used by old apply_op)
  - [x] All `// SAFETY:` comments required on any `unwrap()` calls in non-test code

- [x] Task 2: Remove `StackEngine` compat stub from `src/engine/stack.rs` (AC: enables clean compile)
  - [x] Delete the `StackEngine` struct and its entire `impl` block (lines ~87–163)
  - [x] Delete the `/// Retained temporarily...` comment above it
  - [x] Verify `src/engine/ops.rs` no longer imports `StackEngine` (done in Task 1)

- [x] Task 3: Comprehensive unit tests in `#[cfg(test)]` block in `ops.rs` (AC: 1–11)
  - [x] Test helpers: `fn int(n: i64) -> CalcValue` and `fn float(f: f64) -> CalcValue`
  - [x] Binary ops (happy path): add int+int, sub, mul, div exact-division→Integer, div non-exact→Float, pow int, mod int
  - [x] Binary ops (mixed types): int+float, float+int produces Float
  - [x] Binary ops (errors, atomicity): div by zero → DivisionByZero, stack unchanged (depth+peek verified)
  - [x] Trig (DEG): sin(90°)≈1.0, cos(0°)=1.0, tan(45°)≈1.0
  - [x] Trig (RAD): sin(π/2)≈1.0
  - [x] Trig (GRAD): sin(100 grad)≈1.0
  - [x] Inverse trig: asin(1.0) in DEG≈90.0, acos/atan happy path
  - [x] Inverse trig domain error: asin(2.0) → DomainError, stack unchanged
  - [x] Log ops: ln(e)≈1.0, log10(100)≈2.0, exp(1)≈e, exp10(2)≈100
  - [x] Log domain errors: ln(0) → DomainError, ln(-1) → DomainError, log10(0) → DomainError, stack unchanged
  - [x] Utility: sqrt(4)→2, square(3)→9, reciprocal(4)→0.25, abs(-5)→5, abs(-2.5f)→2.5, negate(3)→-3, negate(-2.5f)→2.5f
  - [x] Sqrt domain: sqrt(-1) → DomainError, stack unchanged
  - [x] Reciprocal domain: reciprocal(0) → DivisionByZero, stack unchanged
  - [x] Factorial: factorial(5)→120, factorial(0)→1, factorial(-1)→DomainError, factorial(float)→NotAnInteger
  - [x] Bitwise binary: AND, OR, XOR, SHL, SHR on integers — correct results
  - [x] Bitwise unary: NOT on integer — correct result
  - [x] Bitwise on float: AND(float,int) → NotAnInteger, NOT(float) → NotAnInteger; stack unchanged
  - [x] Constants: PushPi pushes ≈3.14159, PushE pushes ≈2.71828, PushPhi pushes ≈1.61803
  - [x] Stack ops via apply_op: Swap, Dup, Drop, Rotate, Clear all delegate correctly
  - [x] Stack op underflow via apply_op: returns Err(StackUnderflow)

- [x] Task 4: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — 99 unit tests + 2 integration tests pass

## Dev Notes

### Scope — Files Changed

This story modifies **two files**:
1. `src/engine/ops.rs` — primary: new dispatch, atomic helpers, CalcError migration, StackEngine removal
2. `src/engine/stack.rs` — secondary: remove `StackEngine` compat stub

No other files need changes. `engine/mod.rs` already has all required re-exports.

### Critical: Atomic Operations Pattern

**Architecture requirement (NFR7, NFR8):** Stack state is NEVER partially modified. If an operation returns `Err`, the stack must be identical to before the call.

The old `binary_op` helper in ops.rs **violates atomicity** — it pops both values before calling the computation function, so if computation fails, the values are lost. Story 1.4 MUST fix this.

**Correct atomic binary_op pattern (peek-compute-then-mutate):**
```rust
fn binary_op(
    state: &mut CalcState,
    f: impl Fn(CalcValue, CalcValue) -> Result<CalcValue, CalcError>,
) -> Result<(), CalcError> {
    if state.depth() < 2 {
        return Err(CalcError::StackUnderflow);
    }
    // Peek without mutating
    let n = state.stack.len();
    let b = state.stack[n - 1].clone(); // top (X)
    let a = state.stack[n - 2].clone(); // second (Y)
    // Compute — if Err, nothing has been modified yet
    let result = f(a, b)?;
    // Only mutate on success
    state.pop().expect("SAFETY: depth >= 2 checked above");
    state.pop().expect("SAFETY: depth >= 2 checked above");
    state.push(result);
    Ok(())
}
```

**Correct atomic unary_op pattern:**
```rust
fn unary_op(
    state: &mut CalcState,
    f: impl Fn(CalcValue) -> Result<CalcValue, CalcError>,
) -> Result<(), CalcError> {
    // Peek without mutating
    let a = state.peek().ok_or(CalcError::StackUnderflow)?.clone();
    // Compute — if Err, nothing modified
    let result = f(a)?;
    // Only mutate on success
    state.pop().expect("SAFETY: peeked above means depth >= 1");
    state.push(result);
    Ok(())
}
```

Note: `expect("SAFETY: ...")` is acceptable — it documents why the unwrap is safe and is used only when the SAFETY invariant guarantees infallibility. This satisfies the architecture's no-unwrap-without-SAFETY rule.

### New `apply_op` Signature

The old `apply_op(engine: &mut StackEngine, op: &str) -> Result<String, String>` is being replaced with:

```rust
pub fn apply_op(state: &mut CalcState, op: Op) -> Result<(), CalcError>
```

Dispatch skeleton:
```rust
pub fn apply_op(state: &mut CalcState, op: Op) -> Result<(), CalcError> {
    match op {
        // Binary arithmetic
        Op::Add => binary_op(state, do_add),
        Op::Sub => binary_op(state, do_sub),
        Op::Mul => binary_op(state, do_mul),
        Op::Div => binary_op(state, do_div),
        Op::Pow => binary_op(state, do_pow),
        Op::Mod => binary_op(state, do_mod),
        // Binary bitwise
        Op::And => binary_op(state, do_and),
        Op::Or  => binary_op(state, do_or),
        Op::Xor => binary_op(state, do_xor),
        Op::Shl => binary_op(state, do_shl),
        Op::Shr => binary_op(state, do_shr),
        // Unary
        Op::Negate    => unary_op(state, do_negate),
        Op::Sqrt      => unary_op(state, do_sqrt),
        Op::Square    => unary_op(state, do_sq),
        Op::Reciprocal => unary_op(state, do_reciprocal),
        Op::Abs       => unary_op(state, do_abs),
        Op::Factorial => unary_op(state, do_factorial),
        Op::Not       => unary_op(state, do_not),
        // Trig (capture angle_mode from state before closure)
        Op::Sin  => { let m = state.angle_mode; unary_op(state, |v| do_trig(v, m, f64::sin)) }
        Op::Cos  => { let m = state.angle_mode; unary_op(state, |v| do_trig(v, m, f64::cos)) }
        Op::Tan  => { let m = state.angle_mode; unary_op(state, |v| do_trig(v, m, f64::tan)) }
        Op::Asin => { let m = state.angle_mode; unary_op(state, |v| do_atrig(v, m, f64::asin)) }
        Op::Acos => { let m = state.angle_mode; unary_op(state, |v| do_atrig(v, m, f64::acos)) }
        Op::Atan => { let m = state.angle_mode; unary_op(state, |v| do_atrig(v, m, f64::atan)) }
        // Log/Exp
        Op::Log10 => unary_op(state, do_log10),
        Op::Ln    => unary_op(state, do_ln),
        Op::Exp   => unary_op(state, do_exp),
        Op::Exp10 => unary_op(state, do_exp10),
        // Stack ops — delegate to CalcState methods from Story 1.3
        Op::Swap   => state.swap(),
        Op::Dup    => state.dup(),
        Op::Drop   => state.drop(),
        Op::Rotate => state.rotate(),
        Op::Clear  => { state.clear(); Ok(()) }
        // Constants
        Op::PushPi  => { state.push(constants::pi());    Ok(()) }
        Op::PushE   => { state.push(constants::euler()); Ok(()) }
        Op::PushPhi => { state.push(constants::phi());   Ok(()) }
    }
}
```

### Error Type Migration (`String` → `CalcError`)

All private `do_*` functions must change signature from `Result<CalcValue, String>` to `Result<CalcValue, CalcError>`. The helpers `binary_op` and `unary_op` also change to `Result<(), CalcError>`.

Error mapping:
| Old String | New CalcError |
|---|---|
| `"Division by zero"` | `CalcError::DivisionByZero` |
| `"Modulo by zero"` | `CalcError::DivisionByZero` |
| `"Domain error"` | `CalcError::DomainError("...".to_string())` |
| `"Log requires positive number"` | `CalcError::DomainError("log requires positive number".to_string())` |
| `"Ln requires positive number"` | `CalcError::DomainError("ln requires positive number".to_string())` |
| `"Sqrt requires non-negative number"` | `CalcError::DomainError("sqrt requires non-negative number".to_string())` |
| `"Factorial requires non-negative integer"` | `CalcError::DomainError("factorial requires non-negative integer".to_string())` |
| `"Number too large for factorial"` | `CalcError::DomainError("factorial argument too large".to_string())` |
| `"Factorial argument too large"` | `CalcError::DomainError("factorial argument too large".to_string())` |
| `"Bitwise AND requires integers"` | `CalcError::NotAnInteger` |
| `"Bitwise OR requires integers"` | `CalcError::NotAnInteger` |
| `"Bitwise XOR requires integers"` | `CalcError::NotAnInteger` |
| `"Bitwise NOT requires integer"` | `CalcError::NotAnInteger` |
| `"Shift requires integers"` | `CalcError::NotAnInteger` |
| `"Shift amount too large"` | `CalcError::DomainError("shift amount too large".to_string())` |
| `"Factorial requires a non-negative integer"` (float case) | `CalcError::NotAnInteger` |
| `"Stack underflow"` | `CalcError::StackUnderflow` |

### Bug Fix: `do_abs` Float Branch

The current `do_abs` Float branch has the FBig binary string bug (same bug fixed in Story 1.2):

```rust
// BROKEN — FBig.to_string() returns binary representation, not decimal
CalcValue::Float(x) => {
    let val = x.to_string().parse::<f64>().unwrap_or(0.0);
    Ok(CalcValue::from_f64(val.abs()))
}
```

Fix:
```rust
// CORRECT — use to_f64().value() for decimal float value
CalcValue::Float(x) => {
    let val = x.to_f64().value();
    Ok(CalcValue::from_f64(val.abs()))
}
```

### Missing Operation: `do_negate`

`Op::Negate` is in the enum but no implementation exists. Add:

```rust
fn do_negate(v: CalcValue) -> Result<CalcValue, CalcError> {
    match v {
        CalcValue::Integer(x) => Ok(CalcValue::Integer(-x)),
        CalcValue::Float(x) => Ok(CalcValue::Float(-x)),
    }
}
```

IBig and FBig both implement `Neg` via the `-` prefix operator.

### StackEngine Removal

After Story 1.4, `StackEngine` has no purpose. Remove from `stack.rs`:
- The `/// Retained temporarily so...` comment (line ~86)
- The `#[allow(dead_code)]` attribute (line ~87)
- The `pub struct StackEngine { ... }` block
- The entire `#[allow(dead_code)]` `impl StackEngine { ... }` block

In `ops.rs`:
- Remove `use crate::engine::stack::StackEngine;`

No other files reference `StackEngine`.

### `int_to_fbig` Helper — Keep As-Is

The existing `fn int_to_fbig(n: &IBig) -> FBig` function parses via f64. This loses precision for very large integers, but since trig/log operations go through f64 anyway (architecture says "f64 intermediate for trig/log is acceptable" — NFR3), this is by design. Do NOT change it.

### Trig Angle Mode Capture Pattern

The trig ops need `state.angle_mode` but `unary_op` takes `&mut CalcState`. Capture `angle_mode` as a local copy before passing to `unary_op` (avoid borrow conflicts):

```rust
Op::Sin => {
    let m = state.angle_mode;  // Copy before mutable borrow
    unary_op(state, |v| do_trig(v, m, f64::sin))
}
```

`AngleMode` is `Copy`, so this works with no cloning.

### `do_div` Double Zero-Check Cleanup

The current `do_div` checks zero twice (once via `b.to_f64() == 0.0` at the top, then inside the Integer+Integer arm). The top-level check covers Float arms. The Integer arm's check is also fine. This double-check pattern is acceptable — do NOT restructure it, just convert the error types.

### Test Helper Signatures

Add these to the `#[cfg(test)]` block:

```rust
fn int(n: i64) -> CalcValue {
    CalcValue::Integer(IBig::from(n))
}

fn float(f: f64) -> CalcValue {
    CalcValue::from_f64(f)
}

fn state_with(vals: &[i64]) -> CalcState {
    let mut s = CalcState::new();
    for &v in vals {
        s.push(int(v));
    }
    s
}
```

Use `i64` for `int()` (vs `i32` in Story 1.3 stack tests) to handle larger factorial test values.

### Tolerance for Float Assertions

Float result comparisons must use tolerance, not equality:

```rust
fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-10
}
```

Example: `assert!(approx_eq(result_f64, expected_f64), "got {result_f64}, expected {expected_f64}")`.

### Previous Story Learnings

From Stories 1.1–1.3:
1. `#![allow(dead_code)]` and `#![allow(unused_imports)]` at crate root — unused functions don't generate warnings yet
2. `FBig::to_string()` returns **binary** representation — always use `f.to_f64().value()` for decimal value
3. `IBig` supports `+`, `-`, `*`, `/`, `%`, `&`, `|`, `^`, `!` (bitwise NOT), `<<`, `>>` — all operator traits implemented
4. `FBig` supports `+`, `-`, `*`, `/`, unary `-` — but NOT bitwise operators
5. No `unwrap()` in non-test code without `// SAFETY:` or `.expect("SAFETY: ...")`
6. `cargo clippy -- -D warnings` is a hard gate — run before marking any task complete
7. All error cases must leave stack unchanged — tests must verify `depth()` and `peek()` after every `Err` result
8. `CalcState::stack` is `pub` — direct index access `state.stack[n-1]` is valid in ops.rs for the atomic peek pattern

### References

- Operations: [FR2 binary ops, FR3 unary ops, FR9–13 specific ops, FR14 constants | Source: epics.md#Story 1.4]
- Angle mode conversions: `to_radians()` / `from_radians()` [Source: src/engine/angle.rs]
- Constants: `pi()`, `euler()`, `phi()` [Source: src/engine/constants.rs]
- Error types: `CalcError` variants [Source: src/engine/error.rs]
- Atomic op requirement: [Source: architecture.md#Communication Patterns — "Stack state is NEVER partially modified"]
- Error handling (no unwrap): [Source: architecture.md#Process Patterns]
- f64 intermediate for trig/log: [Source: architecture.md#Requirements Overview, NFR3]
- `Op` enum (30+ variants, already defined): [Source: src/engine/ops.rs:10–54]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

1. `FBig::ZERO` ambiguity (E0283): `FBig` is generic over `RoundingMode`, so `FBig::ZERO` can't be used in `==` without type annotation. Fixed by replacing `y == FBig::ZERO` with `y.to_f64().value() == 0.0` in the Float arms of `do_div`.

### Completion Notes List

1. Rewrote `apply_op` from `(&mut StackEngine, &str) -> Result<String, String>` to `(&mut CalcState, Op) -> Result<(), CalcError>` — typed dispatch, no string matching.
2. Implemented atomic peek-compute-then-mutate pattern in `binary_op` and `unary_op` helpers — stack is never partially modified on error.
3. Added missing `do_negate` (was in `Op` enum but never implemented).
4. Fixed `do_abs` float branch: `x.to_string().parse::<f64>()` → `x.to_f64().value()` (FBig binary string bug, same as Story 1.2).
5. Converted all `do_*` functions from `Result<CalcValue, String>` to `Result<CalcValue, CalcError>` with correct variant mapping.
6. Removed `StackEngine` struct and impl block from `stack.rs` entirely.
7. Removed old `apply_op`, `binary_op` (old), `unary_op` (old), and `op_symbol` from `ops.rs`.
8. `do_factorial` float variant returns `CalcError::NotAnInteger` (not `DomainError`).
9. 58 new unit tests in ops.rs covering all ops, error paths, atomicity, and all 3 angle modes.
10. 99 unit + 2 integration tests all pass; clippy clean; fmt applied.

### File List

- `src/engine/ops.rs`
- `src/engine/stack.rs`
