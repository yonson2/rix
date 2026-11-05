# [119] Secret looks

**Status:** open
**Milestone:** M8
**Depends on:** [114], [115]
**Created:** 2026-11-05
**Closed:** -

## Problem
Hidden goals surprise players, and players talk about them.

## Approach
- Three looks with hidden conditions. The skins screen shows "???" until the player finds
  one; then a notice says "SECRET FOUND".
- The conditions: clear a level at exactly 75.0 % (one decimal); draw one line that
  touches all four walls; end a run on a palindrome score of 1,000 or more (for example
  12,321).
- The looks: one line, one wall and one fill, made in this issue.

## Verification
`cargo test` passes with a test for each condition; `just shot` shows each look.

## Notes
- From the 2026-11-05 brainstorm (D39).
