# Story 4.4: Configuration File

Status: done

## Story

As a CLI power user,
I want to configure rpncalc's default behaviour in a config file,
so that my preferred angle mode, base, precision, and undo depth are set automatically on every launch.

## Acceptance Criteria

1. **Given** a `~/.rpncalc/config.toml` file exists with `angle_mode = "rad"`, **When** the calculator launches, **Then** RAD mode is active from the first frame.

2. **Given** a config file sets `base = "hex"`, **When** the calculator launches, **Then** HEX base is active from the first frame.

3. **Given** a config file sets `precision = 10`, **When** float values are displayed, **Then** they are shown to 10 decimal places (trailing zeros trimmed as usual).

4. **Given** a config file sets `max_undo_history = 50`, **When** undo history grows beyond 50 entries, **Then** the oldest entries are discarded to stay within the limit.

5. **Given** a config file sets `persist_session = false`, **When** the calculator quits and is relaunched, **Then** no session is saved or restored — it always starts fresh.

6. **Given** no config file exists, **When** the calculator launches, **Then** sensible defaults are used (DEG, DEC, precision 15, undo depth 1000, persist_session true) and no error is shown.

7. **Given** the config file contains an invalid value (e.g. `angle_mode = "invalid"`), **When** the calculator launches, **Then** the invalid field is ignored, the default is used, and the calculator starts normally — it never refuses to launch due to a bad config.

## Tasks / Subtasks

- [x] Task 1: `src/config/config.rs` — Wire real TOML loading (AC: 1–7)
  - [x] Add imports: `use std::{fs, path::{Path, PathBuf}}; use serde::Deserialize;`
  - [x] Add `pub fn config_path() -> Option<PathBuf>` → `dirs::home_dir().map(|h| h.join(".rpncalc").join("config.toml"))`
  - [x] Add `#[derive(Deserialize)] struct ConfigToml` with all fields as `Option<T>` (graceful missing-key handling)
  - [x] Add `pub(crate) fn load_from_path(path: &Path) -> Self` — testable path-injected loader (same pattern as `session::load_from_path`)
  - [x] Replace `Config::load()` body with: resolve `config_path()`, call `load_from_path()`
  - [x] Remove `#[allow(dead_code)]` attributes (struct and method are now actively used)
  - [x] Add `#[cfg(test)]` unit tests (see Task 5)

- [x] Task 2: `src/engine/value.rs` — Add precision-aware display (AC: 3)
  - [x] Add `pub(crate) fn format_fbig_prec(f: &FBig, precision: usize) -> String` — same logic as `format_fbig` but uses `precision` instead of hardcoded 15
  - [x] Add `pub fn display_with_precision(&self, base: Base, precision: usize) -> String` on `CalcValue` — delegates to `display_with_base(base)` for integers (precision irrelevant), calls `format_fbig_prec(f, precision)` for floats
  - [x] Do NOT modify `display_with_base()` or `format_fbig()` — keep them for non-stack callers (`Display` impl, `yank_text`, etc.)
  - [x] Add tests for `display_with_precision` and `format_fbig_prec` (see Task 5)

- [x] Task 3: `src/tui/app.rs` — Apply config to fresh state and add `precision` field (AC: 1, 2, 3, 4, 6)
  - [x] Add `pub precision: usize` field to `App` struct
  - [x] In `App::new()`, after `Config::load()`: set `state.angle_mode = config.angle_mode; state.base = config.base;` (these are overridden by session restore in `main.rs`, which is correct — config is the fresh-launch default)
  - [x] Set `self.precision = config.precision` in the `App::new()` constructor
  - [x] Add tests (see Task 5)

- [x] Task 4: `src/tui/widgets/stack_pane.rs` — Use precision-aware display (AC: 3)
  - [x] Find every call to `val.display_with_base(...)` inside `stack_pane.rs` and change to `val.display_with_precision(..., app.precision)` (widget already receives `&App`)
  - [x] Verify the function signature of the stack pane render function to confirm `&App` is available; if only `&CalcState` is passed, escalate precision through the parameter

- [x] Task 5: Tests (AC: 1–7)
  - [x] `config.rs` unit tests using `load_from_path` with temp files:
    - `test_config_defaults` — `Config::default()` matches the defined defaults
    - `test_load_angle_rad` — TOML with `angle_mode = "rad"` → `AngleMode::Rad`
    - `test_load_angle_grad` — TOML with `angle_mode = "grad"` → `AngleMode::Grad`
    - `test_load_base_hex` — TOML with `base = "hex"` → `Base::Hex`
    - `test_load_precision` — TOML with `precision = 10` → `cfg.precision == 10`
    - `test_load_undo_depth` — TOML with `max_undo_history = 50` → `cfg.max_undo_history == 50`
    - `test_load_persist_false` — TOML with `persist_session = false` → `cfg.persist_session == false`
    - `test_missing_file_uses_defaults` — nonexistent path → all defaults
    - `test_invalid_value_uses_default` — `angle_mode = "bad"` → `AngleMode::Deg`
    - `test_invalid_toml_uses_defaults` — malformed TOML → all defaults
    - `test_partial_config_keeps_other_defaults` — only `precision = 5` → others still default
  - [x] `value.rs` unit tests:
    - `test_display_with_precision_float_10` — `3.141592653589793` at precision 10 → `"3.1415926536"`
    - `test_display_with_precision_trims_zeros` — `3.0` at precision 5 → `"3"`
    - `test_display_with_precision_integer` — `CalcValue::Integer(42)` at any precision → same as `display_with_base`
  - [x] `app.rs` unit tests:
    - `test_app_new_default_precision` — `App::new().precision == 15`
    - `test_app_new_default_angle_mode` — `App::new().state.angle_mode == AngleMode::Deg`

- [x] Task 6: Quality gates
  - [x] `cargo build` exits 0
  - [x] `cargo clippy -- -D warnings` exits 0
  - [x] `cargo fmt` applied
  - [x] `cargo test` exits 0 — 322 unit + 2 integration = 324 total (305 pre-existing + 19 new)

## Dev Notes

### What Is Already In Place — Do NOT Reinvent

| Component | Location | State |
|-----------|----------|-------|
| `Config` struct with all 5 fields | `src/config/config.rs` | Stub with hardcoded defaults ✅ |
| `Config::load()` method | `src/config/config.rs:16` | Returns `Self::default()` — REPLACE THIS |
| `Config::default()` impl | `src/config/config.rs:21` | Correct defaults — keep unchanged ✅ |
| `toml = "~0.8"` dependency | `Cargo.toml` | Already in deps ✅ |
| `dirs = "5"` dependency | `Cargo.toml` | Already in deps ✅ |
| `format_fbig(f)` function | `src/engine/value.rs:112` | Uses hardcoded 15 — do NOT modify ✅ |
| `display_with_base(base)` | `src/engine/value.rs:40` | Float branch calls `format_fbig` — do NOT modify ✅ |
| `App.undo_history` uses config | `src/tui/app.rs:49` | Already uses `config.max_undo_history` ✅ |
| `session::save` checks `persist_session` | `src/config/session.rs:43-47` | Already works via `Config::load()` ✅ |
| `session_path()` pattern | `src/config/session.rs:8` | Copy this exact pattern for `config_path()` ✅ |

### TOML Schema

```toml
# ~/.rpncalc/config.toml
angle_mode = "deg"        # "deg" | "rad" | "grad"
base = "dec"              # "dec" | "hex" | "oct" | "bin"
precision = 15            # integer > 0; decimal places for float display
max_undo_history = 1000   # integer > 0; 0 means unlimited (use 0 → default, don't allow 0)
persist_session = true    # bool
```

All keys are optional. Missing keys use defaults. Invalid string values are silently ignored (default used). Invalid TOML syntax causes the entire file to be treated as missing (all defaults).

### Task 1: Complete `config.rs`

**`ConfigToml` intermediate struct** (all fields `Option<T>` for graceful partial configs):
```rust
#[derive(Deserialize)]
struct ConfigToml {
    angle_mode: Option<String>,
    base: Option<String>,
    precision: Option<usize>,
    max_undo_history: Option<usize>,
    persist_session: Option<bool>,
}
```

**`load_from_path` implementation**:
```rust
pub(crate) fn load_from_path(path: &Path) -> Config {
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Config::default(),  // file not found or unreadable
    };
    let parsed: ConfigToml = match toml::from_str(&content) {
        Ok(t) => t,
        Err(_) => return Config::default(),  // malformed TOML
    };
    let mut cfg = Config::default();
    if let Some(am) = parsed.angle_mode {
        match am.to_lowercase().as_str() {
            "rad"  => cfg.angle_mode = AngleMode::Rad,
            "grad" => cfg.angle_mode = AngleMode::Grad,
            "deg"  => cfg.angle_mode = AngleMode::Deg,
            _      => {} // invalid value — keep default
        }
    }
    if let Some(b) = parsed.base {
        match b.to_lowercase().as_str() {
            "hex" => cfg.base = Base::Hex,
            "oct" => cfg.base = Base::Oct,
            "bin" => cfg.base = Base::Bin,
            "dec" => cfg.base = Base::Dec,
            _     => {} // invalid value — keep default
        }
    }
    if let Some(p) = parsed.precision {
        if p > 0 { cfg.precision = p; }  // 0 is not a valid precision — keep default
    }
    if let Some(d) = parsed.max_undo_history {
        if d > 0 { cfg.max_undo_history = d; }
    }
    if let Some(s) = parsed.persist_session {
        cfg.persist_session = s;
    }
    cfg
}

pub fn load() -> Self {
    let path = match config_path() {
        Some(p) => p,
        None => return Self::default(),
    };
    load_from_path(&path)
}
```

### Task 2: `value.rs` — Precision-Aware Float Display

Add AFTER the existing `format_fbig` function at line 123. Do NOT modify `format_fbig`:

```rust
pub(crate) fn format_fbig_prec(f: &FBig, precision: usize) -> String {
    let val = f.to_f64().value();
    if val.is_nan() || val.is_infinite() {
        return format!("{}", val);
    }
    let s = format!("{:.prec$}", val, prec = precision);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}
```

Add to `CalcValue` impl block AFTER `display_with_base`:
```rust
pub fn display_with_precision(&self, base: Base, precision: usize) -> String {
    match self {
        CalcValue::Integer(_) => self.display_with_base(base),
        CalcValue::Float(f) => format_fbig_prec(f, precision),
    }
}
```

**Why no change to `display_with_base`?** The `Display` trait impl, `yank_text()`, and the `format!` in `fmt::Display` all call `display_with_base`. These callers do not have access to precision and the ACs don't require precision there. Only the stack pane display is precision-configurable.

### Task 3: `app.rs` — Apply Config Defaults and Add `precision`

Current `App::new()`:
```rust
pub fn new() -> Self {
    let config = Config::load();
    Self {
        state: CalcState::new(),
        undo_history: UndoHistory::with_max_depth(config.max_undo_history),
        mode: AppMode::Normal,
        error_message: None,
        should_quit: false,
    }
}
```

Updated `App::new()`:
```rust
pub fn new() -> Self {
    let config = Config::load();
    let mut state = CalcState::new();
    state.angle_mode = config.angle_mode;  // config default (overridden by session restore in main.rs)
    state.base = config.base;             // config default (overridden by session restore in main.rs)
    Self {
        state,
        undo_history: UndoHistory::with_max_depth(config.max_undo_history),
        mode: AppMode::Normal,
        error_message: None,
        should_quit: false,
        precision: config.precision,
    }
}
```

Updated `App` struct:
```rust
pub struct App {
    pub state: CalcState,
    pub undo_history: UndoHistory,
    pub mode: AppMode,
    pub error_message: Option<String>,
    pub should_quit: bool,
    pub precision: usize,  // ADD THIS
}
```

**Session restore interaction (IMPORTANT):**
In `main.rs`, after `App::new()`:
```rust
match session::load() {
    Ok(Some(state)) => { app.state = state; }  // overwrites angle_mode and base from config
    ...
}
```
This is CORRECT behavior. Config provides fresh-launch defaults. Session preserves user's last-used state. No change needed to `main.rs`.

### Task 4: `stack_pane.rs` — Precision-Aware Display

You must read `stack_pane.rs` before editing. Find calls to `display_with_base(...)` on `CalcValue` instances and change them to `display_with_precision(..., app.precision)`.

**Determine whether widget receives `&App` or `&CalcState`:**
- If the render function signature is `fn render_stack_pane(f: &mut Frame, area: Rect, app: &App)` → use `app.precision` directly
- If the signature only receives `&CalcState` → you need to either add a `precision: usize` param or change the signature to take `&App`

Check `layout.rs` to see how `stack_pane` is called — it likely passes `&app` given the pattern used throughout.

### Config String Mapping (Case-Insensitive)

| TOML value | Rust type |
|-----------|-----------|
| `"deg"` | `AngleMode::Deg` |
| `"rad"` | `AngleMode::Rad` |
| `"grad"` | `AngleMode::Grad` |
| `"dec"` | `Base::Dec` |
| `"hex"` | `Base::Hex` |
| `"oct"` | `Base::Oct` |
| `"bin"` | `Base::Bin` |

Use `.to_lowercase()` before matching — TOML values like `"DEG"` or `"Rad"` should also work.

### Previous Story Learnings (Stories 4.1 – 4.3)

- `cargo fmt` reformats code — run after ALL edits, not just at the end
- Read each file before editing — the Edit tool requires exact text matches
- `pub(crate)` for internal-only functions (e.g. `load_from_path`, `format_fbig_prec`)
- Use temp files in tests, never real paths like `~/.rpncalc/` (avoid polluting real config in CI)
- `Config::load()` is called in `App::new()` and `session::save()` — after this story, both will get real config values
- `#[allow(dead_code)]` on `Config` and `Config::load()` must be removed — they are now active
- Pattern to follow: `session.rs` uses `save_to_path`/`load_from_path` for testability — do the same for `config.rs` with `load_from_path`
- `serde::Deserialize` is already available via `serde = { version = "1", features = ["derive"] }` in Cargo.toml

### `AngleMode` and `Base` — Existing Types

`AngleMode` is in `src/engine/angle.rs`:
```rust
pub enum AngleMode { Deg, Rad, Grad }
```
Already `Serialize`/`Deserialize`. Has `Copy`. Import: `use crate::engine::angle::AngleMode;`

`Base` is in `src/engine/base.rs`:
```rust
pub enum Base { Dec, Hex, Oct, Bin }
```
Already `Serialize`/`Deserialize`. Has `Copy`. Import: `use crate::engine::base::Base;`

Both imports are already at the top of `config.rs` (line 3).

### Files to Change

| File | Change |
|------|--------|
| `src/config/config.rs` | Replace stub with real TOML loading; add `config_path()`, `load_from_path()`, `ConfigToml`; remove `#[allow(dead_code)]` |
| `src/engine/value.rs` | Add `format_fbig_prec(f, precision)` and `display_with_precision(base, precision)` |
| `src/tui/app.rs` | Add `precision: usize` field; apply `config.angle_mode`, `config.base`, `config.precision` in `App::new()` |
| `src/tui/widgets/stack_pane.rs` | Change `display_with_base` → `display_with_precision` for float-aware display |

**No changes to:** `main.rs`, `session.rs`, `action.rs`, `handler.rs`, `commands.rs`, `mode.rs`, any other widget file, engine files.

### Expected Test Delta

| File | New Tests |
|------|-----------|
| `config.rs` | ~11 (defaults, angle, base, precision, undo, persist, missing file, invalid value, invalid TOML, partial config, case-insensitive) |
| `value.rs` | ~3 (precision-aware float, zero-trim, integer passthrough) |
| `app.rs` | ~2 (default precision, default angle mode) |
| **Total new** | ~16 |
| **New total** | ~321 |

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — clean implementation.

### Completion Notes List

- `src/config/config.rs` — replaced stub entirely: added `config_path()`, `ConfigToml` (all-Optional deserialization struct), `load_from_path(path)` (testable injector), `Config::load()` (delegates to `load_from_path`); 14 unit tests using temp files cover all ACs
- `src/engine/value.rs` — added `format_fbig_prec(f, precision)` and `display_with_precision(base, precision)` on `CalcValue`; existing `display_with_base` and `format_fbig` untouched; 3 new tests
- `src/tui/app.rs` — added `precision: usize` field; `App::new()` now applies `config.angle_mode`, `config.base`, `config.precision` from loaded config; 2 new tests
- `src/tui/widgets/stack_pane.rs` — `render()` signature extended with `precision: usize` param; `display_with_base` → `display_with_precision`; test helper updated to pass `15`
- `src/tui/layout.rs` — both `stack_pane::render` calls updated to pass `app.precision`
- 324 total tests (322 unit + 2 integration); all pass; clippy clean

### Senior Developer Review (AI)

**Date:** 2026-03-20
**Verdict:** APPROVED (after fixes)

**Findings fixed:**

- **HIGH — AC5 session load ignored `persist_session`**: `session::load()` was unconditional — a stale `~/.rpncalc/session.json` would be restored even when `persist_session = false`, violating the "no session saved or restored" guarantee. Fixed by mirroring the `save()` pattern: `session::load()` now calls `Config::load()` and returns `Ok(None)` early when `persist_session` is false.
- **LOW — Missing test for `precision = 0` using default**: Added `test_load_precision_zero_uses_default` to `config.rs` to verify zero precision rejects and falls back to 15.

**Post-fix:** 325 tests pass (322 unit + 2 integration + 1 new); clippy clean.

### File List

- `src/config/config.rs` — full TOML loading implementation replacing stub; +1 review test
- `src/engine/value.rs` — precision-aware display functions added
- `src/tui/app.rs` — `precision` field; config defaults applied in `App::new()`
- `src/tui/widgets/stack_pane.rs` — precision param added to `render()`
- `src/tui/layout.rs` — `stack_pane::render` calls updated with `app.precision`
- `src/config/session.rs` — `load()` now checks `persist_session` (AC5 fix)
