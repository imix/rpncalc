# Notation — Behaviour-Level

Rules governing how numeric values are displayed across all notation modes.

---

## Auto Notation Threshold

When notation mode is `auto`, scientific notation activates based on the magnitude of the value:

- Scientific notation is used when `|value| ≥ 1e10`
- Scientific notation is used when `|value| < 1e-4` and `value ≠ 0`
- Fixed-point notation is used for all other values

This threshold applies to both floats and integers. An integer `≥ 1e10` displays in scientific notation (e.g. `10000000000` → `1e10`); integers below the threshold display unchanged.

**Implication:** any display, formatting, or test code that depends on `auto` mode must use this threshold. The threshold is not configurable — it is a fixed part of the `auto` mode definition.

---

## Notation Mode Defaults and Mode Bar Indicators

- Default notation: `fixed` (no indicator shown in mode bar)
- `sci` mode: mode bar shows `SCI` indicator
- `auto` mode: mode bar shows `AUTO` indicator
- `fixed` mode: no notation indicator (it is the default/silent state)

The mode bar indicator is omitted entirely if it would overlap the mode indicator or settings — the label is never partially shown.
