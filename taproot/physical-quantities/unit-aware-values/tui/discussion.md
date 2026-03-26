# Discussion: tui

## Session
- **Date:** 2026-03-26
- **Skill:** tr-implement

## Pivotal Questions

**1. How should tagged values be stored internally — as a normalized base unit or in the user's unit?**
Two options: (a) always normalize to a base unit (kg, m, Kelvin) on input and convert back on display; (b) store the amount in the user's unit and convert on demand. Option (a) is cleaner for arithmetic (no conversion needed at op time), but temperature normalization to Kelvin is surprising and breaks the "delta arithmetic" model the spec requires. Option (b) is chosen: store in user's unit, convert only when the two operands differ.

**2. What happens with `unitless ÷ tagged` (e.g. `3 ÷ 1.9 oz`)?**
The spec covers `tagged ÷ unitless` (keeps unit) and `tagged ÷ tagged-same-category` (dimensionless result), but not the reverse. Physically `3 / 1.9 oz = 1.578 oz⁻¹` (compound). Since compound units are not user-facing in v1, treating it as an error ("compound unit not supported") is consistent with the spec's stance. This decision is recorded in impl.md Notes.

**3. How does the user type `°F` and `°C` in the terminal?**
The `°` character is not reliably typable in all terminal environments. ASCII aliases `F`/`C` are accepted by both the parser and the `in` command. The display always shows the canonical `°F`/`°C`. This is an input convenience, not a rename of the unit.

## Alternatives Considered
- **Normalize to base unit on input** — rejected: temperature normalization to Kelvin conflicts with delta arithmetic and is confusing to users reading `session.json`.
- **`f64` amount vs `CalcValue` recursive tagged** — recursive (`CalcValue::Tagged(Box<CalcValue>)`) would preserve Integer/Float distinction for tagged values. Rejected as over-engineered: unit quantities are always physical measurements (real numbers), and the entire engine converts to f64 for all arithmetic anyway.
- **`check-if-affected-by` pattern for unit safety** — considered making "unit-safe arithmetic" a cross-cutting DoD rule. Rejected: unit logic is entirely contained in `ops.rs` dispatch; it is not a cross-cutting concern.

## Decision
Store tagged values as `(amount: f64, unit: String)`. Add a `tagged_binary_op` and `tagged_unary_op` intercept at the `apply_op` dispatch level so existing arithmetic closures handle plain values unchanged. Static unit registry with linear search for ~20 entries; affine temperature path separate from linear scale path.

## Open Questions
- Should `floor`/`ceil`/`trunc`/`round` on a tagged value preserve the unit? The spec doesn't cover these explicitly. Implemented as "preserve unit" (e.g. `floor(1.9 oz) = 1 oz`) since rounding is dimensionally valid, but this is an assumption.
- Should the `in` keyword be case-insensitive (`IN g`, `in g`)? Currently case-sensitive (`in` lowercase only). Can be relaxed in a follow-up.
