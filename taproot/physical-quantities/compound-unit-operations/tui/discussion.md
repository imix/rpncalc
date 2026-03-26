# Discussion: tui

## Session
- **Date:** 2026-03-26
- **Skill:** tr-implement

## Pivotal Questions

1. **How to store and display compound unit strings without a structural change to TaggedValue?**
   Keeping `unit: String` as the canonical representation and parsing it on demand (rather than storing a `Vec<(String, i8)>`) was chosen. This avoids a breaking serde change and keeps the data model minimal. The trade-off: `parse_unit_expr_atoms` is called multiple times per arithmetic operation, but these operations are infrequent and short-circuited.

2. **How to handle unit display for derived results (e.g. km/h from 100km ÷ 2h)?**
   The atom-based approach preserves the user's chosen unit abbreviations through arithmetic. When dividing `100 km` by `2 h`, the result atoms are `[("km", 1), ("h", -1)]`, displayed as `km/h`. This is more natural than SI-canonical display (which would give `m/s`). The trade-off: exotic combinations like `ft/min` produce `ft/min` not `m/s` — which is actually desirable.

3. **How to handle Add/Sub between compound units (e.g. m/s + km/h)?**
   Replacing the category check with a dim-equality check enables this naturally. The conversion uses `compound_to_si_scale` to compute scale factors from the atom lists, avoiding any category-specific logic.

## Alternatives Considered

- **Store `Vec<(String, i8)>` in TaggedValue** — rejected because it requires a serde schema change, breaking existing sessions, and adds complexity for a rarely-needed representation.
- **Derive unit display from DimensionVector only** — rejected because it loses the user's chosen unit abbreviations (e.g. displaying `m/s` for what the user typed as `km/h`).
- **Use a separate `CompoundTaggedValue` CalcValue variant** — rejected because it duplicates the entire TaggedValue infrastructure and complicates the ops dispatch without meaningful benefit.

## Decision

Atom-based parsing of unit strings on demand, with `TaggedValue.unit` remaining a String display label. Arithmetic operations parse operand unit strings to atoms, combine them, and format the result. The existing `DimensionVector` from `compound-unit-model` continues to serve as the type-checking oracle; the atom list serves as the display oracle. This separation keeps concerns clean: dims for validation, atoms for display.

## Open Questions
- None
