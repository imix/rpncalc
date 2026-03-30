# Taproot Plan

_Built: 2026-03-30 — 13 items_
_HITL = human decision required · AFK = agent executes autonomously_

## Items

1. pending  [refine]  afk   taproot/specs/state-and-memory/session-persistence/usecase.md
   <!-- Fix `q`→`Q` quit key (C-1 blocker — Main Flow step 1 + AC-1) -->

2. pending  [refine]  afk   taproot/specs/stack-management/push-value/usecase.md
   <!-- Remove `d` from Insert-mode shortcut list (C-2 — `d` is Noop in Normal mode) -->

3. pending  [refine]  afk   taproot/specs/configuration/configure-settings-chord/usecase.md
   <!-- Add AC-16: `X` key is Noop in Normal mode (CG-2 gap) -->

4. pending  [refine]  afk   taproot/specs/discoverability/browse-hints-pane/usecase.md
   <!-- Add AC for PrecisionInput modal layout -->

5. pending  [refine]  afk   taproot/specs/state-and-memory/undo-redo/usecase.md
   <!-- Add RESET command to Alternate Flows + notes -->

6. pending  [refine]  afk   taproot/specs/stack-management/intent.md
   <!-- Reword "2 keypresses" success criterion -->

7. pending  [refine]  hitl  taproot/specs/discoverability/intent.md
   <!-- Clarify "2 context states" — needs design decision on exact wording -->

8. pending  [refine]  afk   taproot/specs/mathematical-operations/intent.md
   <!-- Remove/neutralise "41 operations" count -->

9. pending  [spec]    hitl  taproot/global-truths/key-bindings_behaviour.md
   <!-- Capture `q`=x², `Q`=quit, `w`=√ as a global truth -->

10. pending  [spec]   hitl  taproot/global-truths/register-ops_behaviour.md
    <!-- Capture `S`=peek-store vs `i+STORE`=pop-store as a global truth -->

11. pending  [spec]   hitl  taproot/global-truths/notation_behaviour.md
    <!-- Capture `auto` notation threshold (≥1e10 or <1e-4) as a global truth -->

12. pending  [spec]   hitl  taproot/global-truths/chord-leaders_behaviour.md
    <!-- Capture `r`=`r›` chord leader, `R`=rotate rebinding as a global truth -->

13. pending  [refine] hitl  taproot/settings.yaml
    <!-- Add `.claude` to the validator folder-name ignore list -->
