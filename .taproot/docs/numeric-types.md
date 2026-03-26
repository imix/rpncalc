# Numeric Type Architecture

Rules for how numeric values are stored and displayed in rpncalc. Every
implementation that introduces a new `CalcValue` variant, stores a numeric
amount, or performs arithmetic must be checked against these rules before
coding begins (enforced via `definitionOfReady`).

---

## The core rule

**All numeric amounts stored in `CalcValue` variants must use dashu types
(`IBig` for integers, `FBig` for arbitrary-precision floats) — never `f64`.**

`f64` is only acceptable for:
- Unit scale factors in the static registry (read-only constants, never stored on the stack)
- Display intermediates that are immediately formatted and discarded
- Interop with external APIs that require `f64`

### Why

`f64` has 53 bits of mantissa — ~15–16 significant decimal digits. Operations
on values that cannot be represented exactly (e.g. `3.2`, `30.48`) accumulate
rounding error. After a round-trip conversion (`ft → cm → ft`) the result is
`3.200000000000001` rather than `3.2`. The user sees noise.

dashu's `FBig` uses arbitrary-precision binary floating point. Configured at
128 bits of precision (about 38 significant decimal digits), it keeps values
clean through display precision.

### The failed justification to avoid

> "Using f64 is consistent with the engine — all arithmetic routes through to_f64()."

This is false. `CalcValue::Float` stores `FBig` and never touches `f64` for
display. `to_f64()` exists only for compatibility bridges (e.g. trig functions
via `libm`). Storing stack values as `f64` defeats the precision model.

---

## Display chain

Every `CalcValue` variant must participate in the full display chain:

| Method | Contract |
|--------|---------|
| `display_with_precision(n)` | Format to `n` significant digits using `format_fbig_prec` or equivalent dashu formatter. Never delegate to `format_f64_shortest`. |
| `display_with_notation(n, notation)` | Apply Sci/Auto/Fixed notation via `format_fbig_notation` or equivalent. |
| `display_with_base(base)` | For non-integer variants: decimal only (base only affects `Integer`). |

`format_f64_shortest` may be used only for values that are **not stored on the
stack** — e.g. intermediate display of unit scale factors in error messages.

---

## Conversion arithmetic

When converting between units using f64 scale factors, the result must be
converted back to `FBig` before being stored in a `CalcValue`. The f64
intermediate is acceptable only for the multiplication step itself.

Pattern:
```rust
// f64 scale arithmetic (acceptable — transient)
let result_f64 = amount_f64 * from_scale / to_scale;
// Immediately lift back to FBig for storage
let result_fbig = fbig_from_f64(result_f64);
```

This does not eliminate f64 rounding in the conversion itself, but it ensures
display precision is applied correctly and the value doesn't diverge further
through subsequent operations.

For higher conversion precision in a future iteration: store scale factors as
`FBig` constants rather than `f64` and perform all arithmetic in FBig.

---

## DoR checklist (new numeric value type)

Before any impl that introduces a new `CalcValue` variant or stores a numeric
amount is declared ready, the design must answer:

- [ ] What dashu type holds the amount? (`IBig` or `FBig`)
- [ ] How does `display_with_precision` format it? (must use `format_fbig_prec`)
- [ ] How is f64 interop handled? (f64 only for transient scale arithmetic, immediately lifted back)
- [ ] How is serde handled? (FBig serialises via dashu's serde feature — confirm it round-trips)

---

## History

| Date | Issue | Fix |
|------|-------|-----|
| 2026-03-26 | `parse_value` routed through `f64`, causing `1.223 × 100 = 122.299...` | `parse_decimal_exact` parses directly to `FBig` |
| 2026-03-26 | `TaggedValue.amount: f64` — display showed `3.200000000000001` after `ft → cm → ft` | Changed to `FBig`; conversion lifts f64 result back to FBig |
