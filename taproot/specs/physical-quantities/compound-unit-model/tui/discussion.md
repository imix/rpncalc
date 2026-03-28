# Discussion: tui

## Session
- **Date:** 2026-03-26
- **Skill:** tr-implement

## Pivotal Questions

1. **Should `dim` in `TaggedValue` use `Option<DimensionVector>` or a plain `DimensionVector` with serde default?**
   Plain with `#[serde(default)]` is cleaner — callers never have to unwrap. The zero-dim migration check gives the same session detection without infecting every call site with `Option`.

2. **Should category-based compatibility checks in `ops.rs` be migrated to dim-vector checks now or deferred?**
   Deferred to `compound-unit-operations`. For simple units the results are identical, but changing the error text could break existing tests. The refactor's goal is the data model — keep the logic diff minimal.

3. **Should `s` (second) be added to the unit registry in this refactor or in `compound-unit-operations`?**
   Added here — the registry is the data model artefact and `s` is needed for any speed compound unit. It carries no user-visible surface until the parser supports compound unit expressions.

## Alternatives Considered

- **`Option<DimensionVector>` on `TaggedValue.dim`** — rejected: every arithmetic path would need `Option::unwrap_or_default()`, making compound-unit-operations code noisier with no benefit.
- **Custom serde deserializer for `TaggedValue`** — rejected: more complex than `#[serde(default)]` + post-load validation; the zero-dim check achieves the same spec behaviour with less code.
- **Storing temperature as `{K:1}` with affine offset vs. the existing formula** — retained as-is: the spec says "system retains the affine conversion formula alongside the dimension vector". `to_base: None` continues to signal the affine path; `dim: {K:1}` provides the correct SI classification.

## Decision
`DimensionVector` is a plain struct (not `Option`) with per-field `serde(default)` so compact JSON is written and old sessions deserialise without crashing. Post-load validation detects the old format by checking dim==zero for a unit that should have a non-zero dim, returns a distinct error string, and `main.rs` maps that to the spec-required status message. Arithmetic helpers (`add`, `sub`, `negate`, `halve`) are bundled with the data model now so `compound-unit-operations` has no reason to touch this file again.

## Open Questions
- None — all design decisions were made during this session.
