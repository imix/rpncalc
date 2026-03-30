# Chord Leaders — Behaviour-Level

The current chord leader assignments in Normal mode. These affect multiple behaviours and must be consistent across all specs.

---

## Active Chord Leaders (Normal Mode)

| Key | Chord | Category |
|-----|-------|----------|
| `t` | `t›` | Trig operations |
| `l` | `l›` | Log / exp operations |
| `f` | `f›` | Functions (x², √, 1/x, \|x\|) |
| `r` | `r›` | Rounding and sign operations (FLOOR, CEIL, TRUNC, ROUND, SIGN) |
| `c` | `c›` | Constants (π, e, φ) |
| `C` | `C›` | Configuration (angle mode, base, notation, precision, hex style) |

---

## Key Rebinding: `r` and `R`

The `r` key was rebound from **Rotate** to the `r›` **rounding chord leader** to make the rounding operations accessible. Rotate was moved to `R` (Shift-R):

- `r` → `r›` chord leader (enters rounding/sign submenu)
- `R` (Shift-R) → **rotate**: cycles the top three stack values (1→3, 2→1, 3→2)

**Implication:** any spec that references `r` for rotate is incorrect — rotate is `R`. Any spec that references `r›` as the rounding chord is correct.

---

## Removed Chord Leaders

The following chord leaders were removed and their functions consolidated into `C›`:

- `m›` — previously mode/angle switching → now `C›` angle keys (`d`/`r`/`g`)
- `x›` — previously base switching → now `C›` base keys (`c`/`h`/`o`/`b`)
- `X›` — previously hex style → now `C›` hex style keys (`1`–`4`)

`m`, `x`, and `X` in Normal mode are now Noop (see `key-bindings_behaviour.md`).
