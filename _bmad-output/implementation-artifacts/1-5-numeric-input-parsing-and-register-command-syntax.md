# Story 1.5: Numeric Input Parsing & Register Command Syntax

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a CLI power user,
I want to enter numbers in decimal, hex, octal, and binary formats, and use STORE/RCL command syntax,
So that I can work with any numeric representation and invoke named register operations naturally.

## Acceptance Criteria

1. **Given** a decimal integer is entered (e.g. `42`, `-17`), **When** it is parsed, **Then** it is accepted as a `CalcValue::Integer`.

2. **Given** a decimal float is entered (e.g. `3.14`, `1.5e-3`), **When** it is parsed, **Then** it is accepted as a `CalcValue::Float`.

3. **Given** a hex value is entered (e.g. `0xFF`, `0xff`, `-0x1A`), **When** it is parsed, **Then** it is accepted as the correct `CalcValue::Integer`.

4. **Given** an octal value is entered (e.g. `0o377`, `-0o10`), **When** it is parsed, **Then** it is accepted as the correct `CalcValue::Integer`.

5. **Given** a binary value is entered (e.g. `0b11111111`, `-0b101`), **When** it is parsed, **Then** it is accepted as the correct `CalcValue::Integer`.

6. **Given** digit separators are used (e.g. `1_000_000`, `0xFF_FF`), **When** it is parsed, **Then** they are stripped and the value is parsed correctly.

7. **Given** unrecognizable input is entered, **When** it is parsed, **Then** a `CalcError::InvalidInput(_)` is returned.

8. **Given** a register store command is entered (e.g. `myvar STORE`), **When** `parse_command` is called, **Then** it returns `Ok(Action::StoreRegister("myvar".to_string()))`.

9. **Given** a register recall command is entered (e.g. `r1 RCL`), **When** `parse_command` is called, **Then** it returns `Ok(Action::RecallRegister("r1".to_string()))`.

10. **Given** an unrecognized command is entered, **When** `parse_command` is called, **Then** it returns `Err(CalcError::InvalidInput(_))`.

11. **Given** `parse_value` is called with invalid input, **When** the error is returned, **Then** it is `CalcError::InvalidInput(_)` — not a raw `String` error.

## Tasks / Subtasks

- [x] Task 1: Migrate `parse_value` error type in `src/input/parser.rs` (AC: 1–7, 11)
  - [x] Change return type of `parse_value` from `Result<CalcValue, String>` to `Result<CalcValue, CalcError>`
  - [x] Add import `use crate::engine::error::CalcError;` (or `use crate::engine::CalcError;`)
  - [x] Update all private helper functions (`parse_hex`, `parse_octal`, `parse_binary`, `parse_integer`, `parse_float`) to return `Result<CalcValue, CalcError>` instead of `Result<CalcValue, String>`
  - [x] Replace all `.map_err(|_| format!(...).to_string())` with `.map_err(|_| CalcError::InvalidInput(format!(...)))`
  - [x] Replace `Err("Empty input".to_string())` with `Err(CalcError::InvalidInput("Empty input".to_string()))`
  - [x] Replace `Err("Invalid floating point value".to_string())` with `Err(CalcError::InvalidInput("Invalid floating point value".to_string()))`
  - [x] Replace `Err("Could not convert to float".to_string())` with `Err(CalcError::InvalidInput("Could not convert to float".to_string()))`
  - [x] Update existing 4 tests to still compile after type change (they use `matches!` so only the `Ok(...)` arms need checking — errors may need `.is_err()` or `matches!(_, Err(CalcError::InvalidInput(_)))`)

- [x] Task 2: Implement `parse_command` in `src/input/commands.rs` (AC: 8–10)
  - [x] Remove the `TODO` comment
  - [x] Remove the `#[allow(dead_code)]` attribute (function will now be used in tests)
  - [x] Implement the function body: split input on whitespace, match 2-token patterns `[name, "STORE"]` → `Action::StoreRegister`, `[name, "RCL"]` → `Action::RecallRegister`, all else → `Err(CalcError::InvalidInput(...))`
  - [x] Register name must be non-empty (enforced naturally by 2-token match)

- [x] Task 3: Comprehensive unit tests in `src/input/parser.rs` and `src/input/commands.rs` (AC: 1–11)

  **`parser.rs` tests** (extend existing `#[cfg(test)]` block):
  - [x] `test_parse_integer_positive` — `"42"` → `Ok(CalcValue::Integer(_))` with value 42
  - [x] `test_parse_integer_negative` — `"-17"` → `Ok(CalcValue::Integer(_))` with value -17
  - [x] `test_parse_integer_zero` — `"0"` → `Ok(CalcValue::Integer(_))` with value 0
  - [x] `test_parse_float_decimal` — `"3.14"` → `Ok(CalcValue::Float(_))`
  - [x] `test_parse_float_scientific` — `"1.5e-3"` → `Ok(CalcValue::Float(_))`
  - [x] `test_parse_float_scientific_positive_exp` — `"1e10"` → `Ok(CalcValue::Float(_))`
  - [x] `test_parse_hex_uppercase` — `"0xFF"` → `Ok(CalcValue::Integer(_))` with value 255
  - [x] `test_parse_hex_lowercase` — `"0xff"` → `Ok(CalcValue::Integer(_))` with value 255
  - [x] `test_parse_hex_prefix_uppercase` — `"0XFF"` → `Ok(CalcValue::Integer(_))` with value 255
  - [x] `test_parse_hex_negative` — `"-0xFF"` → `Ok(CalcValue::Integer(_))` with value -255
  - [x] `test_parse_octal` — `"0o377"` → `Ok(CalcValue::Integer(_))` with value 255
  - [x] `test_parse_octal_prefix_uppercase` — `"0O377"` → `Ok(CalcValue::Integer(_))` with value 255
  - [x] `test_parse_octal_negative` — `"-0o10"` → `Ok(CalcValue::Integer(_))` with value -8
  - [x] `test_parse_binary` — `"0b11111111"` → `Ok(CalcValue::Integer(_))` with value 255
  - [x] `test_parse_binary_prefix_uppercase` — `"0B101"` → `Ok(CalcValue::Integer(_))` with value 5
  - [x] `test_parse_binary_negative` — `"-0b101"` → `Ok(CalcValue::Integer(_))` with value -5
  - [x] `test_parse_digit_separators_integer` — `"1_000_000"` → `Ok(CalcValue::Integer(_))` with value 1_000_000
  - [x] `test_parse_digit_separators_hex` — `"0xFF_FF"` → `Ok(CalcValue::Integer(_))` with value 65535
  - [x] `test_parse_empty_string` — `""` → `Err(CalcError::InvalidInput(_))`
  - [x] `test_parse_garbage` — `"abc"` → `Err(CalcError::InvalidInput(_))`
  - [x] `test_parse_invalid_hex` — `"0xGG"` → `Err(CalcError::InvalidInput(_))`
  - [x] `test_parse_invalid_octal` — `"0o99"` → `Err(CalcError::InvalidInput(_))`
  - [x] `test_parse_invalid_binary` — `"0b2"` → `Err(CalcError::InvalidInput(_))`

  **`commands.rs` tests** (add `#[cfg(test)]` block):
  - [x] `test_store_command_simple` — `"myvar STORE"` → `Ok(Action::StoreRegister("myvar".to_string()))`
  - [x] `test_store_command_alphanumeric` — `"r1 STORE"` → `Ok(Action::StoreRegister("r1".to_string()))`
  - [x] `test_rcl_command_simple` — `"myvar RCL"` → `Ok(Action::RecallRegister("myvar".to_string()))`
  - [x] `test_rcl_command_alphanumeric` — `"r1 RCL"` → `Ok(Action::RecallRegister("r1".to_string()))`
  - [x] `test_unknown_command_single_word` — `"STORE"` (no name) → `Err(CalcError::InvalidInput(_))`
  - [x] `test_unknown_command_garbage` — `"not a command"` → `Err(CalcError::InvalidInput(_))`
  - [x] `test_unknown_command_empty` — `""` → `Err(CalcError::InvalidInput(_))`
  - [x] `test_unknown_command_wrong_verb` — `"myvar SAVE"` → `Err(CalcError::InvalidInput(_))`

- [x] Task 4: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — all existing tests plus new tests pass

## Dev Notes

### Scope — Two Files

This story touches **exactly two files**:
- `src/input/parser.rs` — migrate error type + expand tests
- `src/input/commands.rs` — implement `parse_command` + add tests

No other files need changing:
- `src/input/action.rs` — `StoreRegister(String)` and `RecallRegister(String)` are already defined
- `src/input/mod.rs` — no changes needed
- `src/engine/` — no changes needed

### Task 1: Migrating `parse_value` Error Type

The current signature is `pub fn parse_value(input: &str) -> Result<CalcValue, String>`.

The architecture specifies parsers must return `Result<_, CalcError>` — no raw `String` errors. Change the return type to:
```rust
pub fn parse_value(input: &str) -> Result<CalcValue, CalcError>
```

Add the import at the top of `parser.rs`:
```rust
use crate::engine::CalcError;
```

Then update every private helper to return `Result<CalcValue, CalcError>` and change all error mappings. Example transformation:

```rust
// BEFORE
fn parse_hex(s: &str) -> Result<CalcValue, String> {
    IBig::from_str_radix(s, 16)
        .map(CalcValue::Integer)
        .map_err(|_| format!("Invalid hex number: 0x{}", s))
}

// AFTER
fn parse_hex(s: &str) -> Result<CalcValue, CalcError> {
    IBig::from_str_radix(s, 16)
        .map(CalcValue::Integer)
        .map_err(|_| CalcError::InvalidInput(format!("Invalid hex number: 0x{}", s)))
}
```

Apply the same transformation to `parse_octal`, `parse_binary`, `parse_integer`, and `parse_float`. Also fix the two inline errors in `parse_value` itself (`"Empty input"`) and in `parse_float` (`"Invalid floating point value"` and `"Could not convert to float"`).

### Task 2: Implementing `parse_command`

The full implementation for `commands.rs`:

```rust
use crate::engine::CalcError;
use crate::input::action::Action;

pub fn parse_command(input: &str) -> Result<Action, CalcError> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    match parts.as_slice() {
        [name, "STORE"] => Ok(Action::StoreRegister(name.to_string())),
        [name, "RCL"] => Ok(Action::RecallRegister(name.to_string())),
        _ => Err(CalcError::InvalidInput(format!(
            "Unknown command: {}",
            input
        ))),
    }
}
```

Note: the `#[allow(dead_code)]` attribute on the function stub must be **removed** once the implementation is active (the function is called from tests). The `TODO` comment should also be removed.

### Test Verification of Values

For parser tests that check the actual parsed value (not just `Ok(CalcValue::Integer(_))`), use `dashu::integer::IBig` in the test helpers:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::CalcError;
    use dashu::integer::IBig;

    fn int_val(n: i64) -> CalcValue {
        CalcValue::Integer(IBig::from(n))
    }

    #[test]
    fn test_parse_integer_positive() {
        assert_eq!(parse_value("42"), Ok(int_val(42)));
    }

    #[test]
    fn test_parse_hex_uppercase() {
        assert_eq!(parse_value("0xFF"), Ok(int_val(255)));
    }
    // ...
}
```

For error tests:
```rust
#[test]
fn test_parse_empty_string() {
    assert!(matches!(parse_value(""), Err(CalcError::InvalidInput(_))));
}
```

For `commands.rs` tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::CalcError;
    use crate::input::action::Action;

    #[test]
    fn test_store_command_simple() {
        assert_eq!(
            parse_command("myvar STORE"),
            Ok(Action::StoreRegister("myvar".to_string()))
        );
    }

    #[test]
    fn test_unknown_command_empty() {
        assert!(matches!(
            parse_command(""),
            Err(CalcError::InvalidInput(_))
        ));
    }
}
```

### `Action` PartialEq

The `Action` enum in `src/input/action.rs` already derives `PartialEq` (needed for `assert_eq!` in tests). If it does not, add `#[derive(PartialEq)]`. Check before writing tests that use `assert_eq!`.

### Register Name Validation

The `parse_command` function accepts **any non-empty, non-whitespace token** as a register name — no alphanumeric-only restriction. The name is just the first whitespace-delimited token before `STORE` or `RCL`. Names like `r1`, `myvar`, `x`, `temp_1` are all valid. No need to validate further in this story — register storage/retrieval validation is Story 4.1.

### Existing Test Compatibility

The 4 existing tests in `parser.rs` use `matches!` macro with `Ok(CalcValue::Integer(_))` / `Ok(CalcValue::Float(_))` patterns — these do not inspect the `Err` type, so they will compile unchanged after the type migration. However, you may replace them with the new more-precise tests from Task 3 to avoid duplication. Either approach is acceptable.

### No Integration Tests Needed

This story operates on pure string→value/action parsing. The unit tests in `parser.rs` and `commands.rs` provide full coverage. No new integration tests in `tests/` are required.

### Previous Story Learnings

From Stories 1.1–1.4:
1. `#![allow(dead_code)]` and `#![allow(unused_imports)]` are at crate root — newly implemented functions not yet called from non-test code will not generate warnings
2. `cargo clippy -- -D warnings` is a hard gate — run before marking any task complete
3. `CalcError` is re-exported as `crate::engine::CalcError` — prefer the re-export path for brevity
4. `IBig::from(n)` works for `i32`, `i64`, etc.
5. `FBig` must NOT be compared with `==` against `FBig::ZERO` directly due to E0283 (generic RoundingMode); use `.to_f64().value() == 0.0` pattern if needed

### References

- Input parsing: [FR1 decimal/hex/octal/binary | Source: epics.md#Story 1.5]
- Register commands: [FR23–26, FR40 STORE/RCL syntax | Source: epics.md#Story 1.5]
- `parse_command` signature: [Source: architecture.md#commands.rs]
- `Action` enum: [Source: src/input/action.rs — StoreRegister, RecallRegister already defined]
- Error types: [Source: architecture.md — parsers return Result<_, CalcError>]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

1. `CalcError` did not derive `PartialEq` — `assert_eq!` in tests comparing `Result<_, CalcError>` failed with E0369. Fixed by adding `PartialEq` to the `#[derive(...)]` on `CalcError` in `src/engine/error.rs`. This was also strictly correct from an architecture standpoint (errors should be comparable).

### Completion Notes List

1. Migrated `parse_value` return type from `Result<CalcValue, String>` to `Result<CalcValue, CalcError>` in `src/input/parser.rs`. Used `use crate::engine::CalcError;` re-export path.
2. All 5 private helpers (`parse_hex`, `parse_octal`, `parse_binary`, `parse_integer`, `parse_float`) updated to return `Result<CalcValue, CalcError>`. All `map_err` closures now wrap in `CalcError::InvalidInput(...)`.
3. Implemented `parse_command` in `src/input/commands.rs` using `split_whitespace().collect()` + slice pattern matching on `[name, "STORE"]` / `[name, "RCL"]`.
4. Added `#[derive(PartialEq)]` to `CalcError` in `src/engine/error.rs` (required for `assert_eq!` in tests; also correct architecture).
5. 23 parser tests + 8 command tests added (31 new tests total). Old 4 parser tests replaced by the new comprehensive set.
6. All quality gates: `cargo build` ✓, `cargo clippy -- -D warnings` ✓, `cargo fmt` ✓, `cargo test` 129 unit + 2 integration = 131 total, all pass.

## File List

- `src/input/parser.rs`
- `src/input/commands.rs`
- `src/engine/error.rs`

## Change Log

- Migrated `parse_value` error type `String` → `CalcError::InvalidInput` in `src/input/parser.rs` (Date: 2026-03-18)
- Implemented `parse_command` for STORE/RCL register syntax in `src/input/commands.rs` (Date: 2026-03-18)
- Added `PartialEq` derive to `CalcError` in `src/engine/error.rs` (Date: 2026-03-18)
