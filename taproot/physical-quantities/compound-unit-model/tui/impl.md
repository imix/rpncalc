# Implementation: tui

## Behaviour
../usecase.md

## Design Decisions
- **`DimensionVector` as a struct with 7 `i8` fields**: one per SI base dimension (kg, m, s, A, K, mol, cd). `i8` is sufficient — practical exponents stay within ±8. Derived with `Default` (all zeros = dimensionless). Serde uses `#[serde(default)]` per field and `skip_serializing_if = "is_zero"` to keep session JSON compact (only non-zero exponents written).
- **`TaggedValue.dim: DimensionVector` with `#[serde(default)]`**: allows old session files (no `dim` field) to deserialise without crashing. Zero-dim mismatch (unit is e.g. `"oz"` but dim is all-zeros) is detected post-deserialisation and triggers the migration path.
- **Session migration via post-load validation**: `load_from_path` checks whether any `TaggedValue` in the loaded state has a zero dim vector while its unit maps to a non-zero dim in the registry. If yes, returns `Err("unit format updated")`. `main.rs` maps this to the status message `"session reset — unit format updated"`. All other deserialisation errors continue to show the existing generic corruption message.
- **Dim forwarded unchanged through `ops.rs`**: all 4 `TaggedValue { ... }` struct literals in `tagged_binary_op`/`tagged_unary_op` now include `dim: tagged.dim.clone()`. No arithmetic on exponents — that is deferred to `compound-unit-operations`.
- **`DimensionVector` arithmetic helpers added now**: `add`, `sub`, `negate`, `halve` are part of the data model and included here so `compound-unit-operations` can use them without a further refactor to this file.
- **`UnitCategory` retained**: category-based compatibility checks in `ops.rs` are unchanged. Compound-unit-operations will migrate to dim-vector checks.
- **`s` (second) added to registry**: `compound-unit-operations` needs a time unit. Added as a new linear unit with `to_base: Some("1")` and `dim: {s:1}`. No new user-facing parse/convert surface for this refactor — it becomes visible only when compound-unit-operations is implemented.

## Source Files
- `src/engine/units.rs` — add `DimensionVector` struct, `dim` field on `Unit` and `TaggedValue`, arithmetic helpers, `s` unit entry
- `src/engine/ops.rs` — forward `dim` in all 4 `TaggedValue` struct literals
- `src/config/session.rs` — add `has_old_format_tagged_values` validator and migration error path
- `src/main.rs` — add match arm for `"unit format updated"` migration message

## Commits
- (auto-linked by taproot link-commits)

## Tests
- `src/engine/units.rs` — `test_registry_si_dimensions` (AC-2: oz/lb/g/kg→{kg:1}, all length units→{m:1}, °F/°C→{K:1}, s→{s:1}), `test_tagged_value_dim_serde_roundtrip` (AC-3: compound vector {m:1,s:-1} survives JSON round-trip), `test_dimension_vector_arithmetic` (add/sub/negate/halve helpers)
- `src/config/session.rs` — `test_old_format_session_migration` (AC-4: session with zero-dim TaggedValue returns migration error), `test_new_format_session_loads_normally` (AC-5)

## DoR Resolutions
- condition: hints-spec | note: no changes to hints_pane.rs and no new AppMode introduced — not affected | resolved: 2026-03-26
- condition: numeric-types | note: DimensionVector uses i8 (dimensional metadata, not a numeric amount on the stack); no new CalcValue variant introduced; TaggedValue.amount remains FBig unchanged — not affected | resolved: 2026-03-26

## Status
- **State:** in-progress
- **Created:** 2026-03-26
- **Last verified:** 2026-03-26
