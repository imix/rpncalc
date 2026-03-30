# Register Operations — Behaviour-Level

Rules governing how named register storage works across all behaviours.

---

## Two Store Paths: Pop vs Peek

Named register storage has two distinct entry points with different stack semantics:

**`i` + `<name> STORE` (Alpha mode — pop semantics)**
- User enters Alpha mode via `i`
- Types `<name> STORE` and presses Enter
- Position 1 is **popped** from the stack and written to the register
- Stack depth decreases by 1

**`S` key (AlphaStore mode — peek semantics)**
- User presses `S` from Normal mode (enters AlphaStore sub-mode)
- Types the register name and presses Enter
- Position 1 is **copied** to the register without being removed from the stack
- Stack depth unchanged

**Implication:** the two paths produce different stack depths after execution. Specs and implementations that reference register storage must be explicit about which path they describe. The `S` (peek) path is the more common quick-store shortcut; the Alpha `STORE` (pop) path is the deliberate pop-and-store form.

Both paths are undo-able — undo restores the prior register state and, for the pop path, also restores position 1.
