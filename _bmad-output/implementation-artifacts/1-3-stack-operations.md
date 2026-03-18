# Story 1.3: Stack Operations

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a CLI power user,
I want to push values onto the stack and manipulate them with fundamental stack operations,
So that I can build up operands and manage stack depth during calculations.

## Acceptance Criteria

1. **Given** a value is pushed onto the stack, **When** the stack is inspected, **Then** the new value is at the top and the previous contents are preserved below it.

2. **Given** two or more values on the stack, **When** swap is applied, **Then** the top two values exchange positions and stack depth is unchanged.

3. **Given** one or more values on the stack, **When** dup is applied, **Then** the top value is duplicated and stack depth increases by one.

4. **Given** one or more values on the stack, **When** drop is applied, **Then** the top value is removed and stack depth decreases by one.

5. **Given** three or more values on the stack, **When** rotate is applied, **Then** the top three values cycle: the top moves to third position, second moves to top, third moves to second.

6. **Given** any stack, **When** clear is applied, **Then** the stack is empty.

7. **Given** an insufficient stack for an operation (e.g. swap with one item, rotate with two), **When** the operation is attempted, **Then** the stack is completely unchanged and a `CalcError::StackUnderflow` is returned.

## Tasks / Subtasks

- [x] Task 1: Add stack manipulation methods to `CalcState` in `src/engine/stack.rs` (AC: 1–7)
  - [x] Add `pub fn push(&mut self, val: CalcValue)` — infallible, no return value
  - [x] Add `pub fn pop(&mut self) -> Result<CalcValue, CalcError>` — returns top value or `StackUnderflow`
  - [x] Add `pub fn swap(&mut self) -> Result<(), CalcError>` — requires `depth() >= 2`
  - [x] Add `pub fn dup(&mut self) -> Result<(), CalcError>` — requires `depth() >= 1`
  - [x] Add `pub fn drop(&mut self) -> Result<(), CalcError>` — requires `depth() >= 1`; delegates to `pop().map(|_| ())`
  - [x] Add `pub fn rotate(&mut self) -> Result<(), CalcError>` — requires `depth() >= 3`; roll-down direction per AC5 (see Dev Notes for exact algorithm)
  - [x] Add `pub fn clear(&mut self)` — infallible, calls `self.stack.clear()`
  - [x] All error cases use `CalcError::StackUnderflow` (already in `engine/error.rs`)

- [x] Task 2: Comprehensive unit tests in the `#[cfg(test)]` block at the bottom of `stack.rs` (AC: 1–7)
  - [x] `push` adds to top; existing items shift down; `depth()` increases
  - [x] `pop` returns the top value and removes it; `pop` on empty returns `Err(CalcError::StackUnderflow)`
  - [x] `swap` exchanges top two; `depth()` unchanged; deeper items unaffected
  - [x] `swap` with < 2 items returns `Err(CalcError::StackUnderflow)`; stack unchanged
  - [x] `dup` duplicates top; `depth()` increases by 1
  - [x] `dup` on empty stack returns `Err(CalcError::StackUnderflow)`
  - [x] `drop` removes top; `depth()` decreases by 1
  - [x] `drop` on empty stack returns `Err(CalcError::StackUnderflow)`
  - [x] `rotate` on `[1, 2, 3]` (3=top) produces `[3, 1, 2]` (2=new top) per AC5
  - [x] `rotate` with < 3 items returns `Err(CalcError::StackUnderflow)`; stack unchanged
  - [x] `clear` empties a populated stack; `clear` on an already-empty stack is a no-op (no error)
  - [x] Error path atomicity: confirm `depth()` and `peek()` unchanged after every `Err` result

- [x] Task 3: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — all 41 unit tests + 2 integration tests pass

## Dev Notes

### Scope — Only `src/engine/stack.rs` Changes

This story touches **exactly one file**. No other file changes:
- No ops.rs, no mod.rs, no error.rs, no value.rs
- No dispatch function — that is Story 1.4's job
- No TUI wiring — that is Story 2.1's job
- Do NOT remove or modify the `StackEngine` compat stub

Add all 7 new methods inside the existing `impl CalcState { ... }` block (between `peek()` and the closing brace of that impl, before the `StackEngine` struct).

### Required Import (Already Present)

`use crate::engine::error::CalcError;` — **not imported in stack.rs yet**. Add it at the top. The type is also re-exported as `crate::engine::CalcError`.

### Method Implementations

```rust
pub fn push(&mut self, val: CalcValue) {
    self.stack.push(val);
}

pub fn pop(&mut self) -> Result<CalcValue, CalcError> {
    self.stack.pop().ok_or(CalcError::StackUnderflow)
}

pub fn swap(&mut self) -> Result<(), CalcError> {
    if self.stack.len() < 2 {
        return Err(CalcError::StackUnderflow);
    }
    let n = self.stack.len();
    self.stack.swap(n - 1, n - 2);
    Ok(())
}

pub fn dup(&mut self) -> Result<(), CalcError> {
    let top = self.stack.last().ok_or(CalcError::StackUnderflow)?.clone();
    self.stack.push(top);
    Ok(())
}

pub fn drop(&mut self) -> Result<(), CalcError> {
    self.pop().map(|_| ())
}

pub fn rotate(&mut self) -> Result<(), CalcError> {
    if self.stack.len() < 3 {
        return Err(CalcError::StackUnderflow);
    }
    let n = self.stack.len();
    let x = self.stack[n - 1].clone(); // top (X)
    let y = self.stack[n - 2].clone(); // second (Y)
    let z = self.stack[n - 3].clone(); // third (Z)
    self.stack[n - 3] = x;             // old top → Z position
    self.stack[n - 2] = z;             // old Z → Y position
    self.stack[n - 1] = y;             // old Y → new top (X)
    Ok(())
}

pub fn clear(&mut self) {
    self.stack.clear();
}
```

### ⚠️ Rotate Direction — Critical, Do Not Copy StackEngine

The `StackEngine.rotate()` in the same file implements the **WRONG direction**. Do not copy it.

**Correct rotation per AC5 (roll-down):**
- Before: `[..., Z, Y, X]` — X is top
- After: `[..., X, Z, Y]` — Y is new top

Trace with `[1, 2, 3]` (1=Z, 2=Y, 3=X/top):
- `stack[n-3]` ← 3 (old X → new Z)
- `stack[n-2]` ← 1 (old Z → new Y)
- `stack[n-1]` ← 2 (old Y → new X, new top)
- Result: `[3, 1, 2]` → `peek()` = 2 ✓

Verify: "top (3) moves to third position" ✓ | "second (2) moves to top" ✓ | "third (1) moves to second" ✓

### Error Atomicity Rule (Architecture Requirement)

Architecture mandates: **operations are transactional — never leave partial state**.

For stack ops, this is naturally satisfied because:
- All depth checks happen BEFORE any mutation
- If `Err` is returned, no `self.stack` fields have been modified
- Tests MUST verify `depth()` and `peek()` are unchanged after every `Err` result

### Test Helpers

Add a private helper in the test module:
```rust
fn int(n: i32) -> CalcValue {
    CalcValue::Integer(IBig::from(n))
}
```

Test imports needed:
```rust
use super::*;
use crate::engine::{error::CalcError, value::CalcValue};
use dashu::integer::IBig;
```

### `drop` Method Naming

Rust allows `fn drop(&mut self) -> Result<(), CalcError>` as an inherent method — this does **not** conflict with the `Drop` trait's `fn drop(&mut self)` (different signature). The `StackEngine` already uses this name in the same file without clippy issues. Use `drop` for consistency with FR6 and RPN convention.

### StackEngine Compat Stub — Do Not Touch

`StackEngine` remains in the file as `#[allow(dead_code)]`. Story 1.4 will remove it. Do not:
- Copy its `rotate()` (wrong direction)
- Upgrade its error types (that would be Story 1.4 scope)
- Remove or modify it

### What Is NOT In Scope

- No `apply_stack_op(state, op)` dispatch function — Story 1.4 builds the full `Op` dispatch
- No changes to `engine/mod.rs` — the re-exports already cover everything
- No `push` in Action handling — Story 2.1 wires `App::apply(Action::Push(v))` → `state.push(v)`
- No HexStyle-aware display changes — still deferred to Story 3.3

### Previous Story Learnings

From Stories 1.1 and 1.2:
1. `#![allow(dead_code)]` and `#![allow(unused_imports)]` are at crate root — new public methods on `CalcState` will not generate warnings even though nothing calls them yet
2. `CalcError` is in `engine/error.rs`, re-exported as `engine::CalcError` — use the direct path `use crate::engine::error::CalcError;` in stack.rs
3. No `unwrap()` in non-test code — use `.ok_or(CalcError::StackUnderflow)` or `?`
4. `cargo clippy -- -D warnings` is a hard gate — run before marking any task complete
5. `dashu::integer::IBig::from(n)` works for `i32`, `i64`, `u32`, etc.

### References

- Stack operations: [FR4 swap, FR5 dup, FR6 drop, FR7 rotate, FR8 clear | Source: epics.md#Story 1.3]
- CalcState ownership: [Source: architecture.md#Calculator State Ownership]
- Error handling (no unwrap, Result throughout): [Source: architecture.md#Error Handling Strategy]
- Operation atomicity: [Source: architecture.md#Communication Patterns — "Stack state is NEVER partially modified"]
- Test placement (co-located #[cfg(test)]): [Source: architecture.md#Structure Patterns]
- Stack terminology (last() = X = top): [Source: architecture.md#Naming Patterns]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — implementation matched story spec exactly; no debug issues encountered.

### Completion Notes List

1. Added `use crate::engine::error::CalcError;` import to `stack.rs` (was not present before).
2. All 7 CalcState methods implemented exactly per Dev Notes spec.
3. Rotate direction verified correct (roll-down): `[1,2,3]` → `[3,1,2]`, new top = 2. Did NOT copy StackEngine.rotate() which has the opposite direction.
4. StackEngine compat stub left completely unchanged per story scope rules.
5. 25 unit tests added covering all happy paths, underflow errors, and error atomicity (depth/peek unchanged after Err).
6. All quality gates passed: `cargo build`, `cargo clippy -- -D warnings`, `cargo fmt`, `cargo test` (41 unit + 2 integration).

### File List

- `src/engine/stack.rs`
