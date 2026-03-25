# Intent: Professional Visual Presentation

## Goal
Enable users to experience a visually cohesive, professional TUI that
reflects the quality of the tool — through consistent panel framing, a
coherent color accent, and a clear visual hierarchy that guides the eye
without getting in the way of computation.

## Stakeholders
- **CLI power user**: uses rpnpad daily and wants a tool that looks as
  polished as btop or lazygit — consistent borders, readable panel titles,
  and a professional color scheme that makes the tool feel intentional
- **Developer**: wants the visual design to be systematic, not ad-hoc —
  so future widgets follow the same conventions without extra thought

## Success Criteria
- [ ] All visible panels have matching border styles and titled blocks
- [ ] A single color accent (cyan) is applied consistently to structural
      chrome (panel titles, outer border title) and nowhere else
- [ ] The mode/status bar is visually separated from the main content area
- [ ] The outer border carries the app name as a title
- [ ] A user comparing a screenshot to btop would say "same genre"

## Constraints
- Must stay within ratatui's widget model — no raw terminal escape codes
- No performance regression: frame rendering must remain imperceptible
- Visual changes must not break any existing layout tests

## Behaviours <!-- taproot-managed -->
- [User sees a visually cohesive TUI layout](./polish-visual-style/usecase.md)


## Status
- **State:** active
- **Created:** 2026-03-24
- **Last reviewed:** 2026-03-24
