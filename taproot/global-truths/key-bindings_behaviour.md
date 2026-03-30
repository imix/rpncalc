# Key Bindings — Behaviour-Level

Direct-key bindings that affect multiple behaviours and must be consistent across all specs.

---

## Quit, Square, and Square Root Keys

The following bindings are in effect across the application:

- `q` → **x²** (square): replaces the top stack value with its square; unary operation
- `Q` (Shift-Q) → **quit**: exits rpnpad and triggers a session save (equivalent to SIGTERM)
- `w` → **√** (square root): replaces the top stack value with its square root; unary operation

These bindings were established by the `direct-common-functions` behaviour, which rebound `q` from quit to x² and moved quit to `Q`. Any spec that references quitting rpnpad must use `Q`, not `q`.

**Implication for specs:** references to "the user quits with `q`" are incorrect — the quit key is `Q`. The `q` key executes x².

---

## X, m, and x Keys Are Noop in Normal Mode

After the removal of the `m›`, `x›`, and `X›` chord leaders (consolidated into `C›`):

- `m` in Normal mode → **Noop** (no action, no error)
- `x` in Normal mode → **Noop** (no action, no error)
- `X` in Normal mode → **Noop** (no action, no error)

These keys may be repurposed in future; that decision is out of scope.
