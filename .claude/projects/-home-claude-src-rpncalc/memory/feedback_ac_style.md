---
name: ac-style-feedback
description: Feedback on acceptance criteria writing style for rpncalc epics/stories
type: feedback
---

Acceptance criteria must describe observable system behavior, not implementation details.

**Why:** ACs are behavior specs, not code specs — function names, struct names, type names, and file paths don't belong there.

**How to apply:**
- Never mention function names (e.g., parse_value, handle_key), struct names (CalcValue, CalcState), file paths (engine/stack.rs), or crate names in ACs
- Focus on what the user or system observes: input → action → outcome
- Tech constraints (clippy, fmt, test placement) belong in a "Dev Notes" section, not ACs
- ACs should be testable from a black-box perspective: given a condition, when something happens, then a visible result occurs
