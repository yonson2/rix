# [113] Records: lifetime totals and the next unlock

**Status:** open
**Milestone:** M8
**Depends on:** [111]
**Created:** 2026-11-05
**Closed:** -

## Problem
The save counts runs, claims, kills, levels and play time, but no screen shows them. The
unlocks for games played ([114]) work only if the player sees how close the next one is.

## Approach
- The RECORDS screen ([111]) gets the lifetime totals: games, claims, kills, levels
  cleared, play time, best score, best level.
- Under them, the next unlock and its progress, for example
  "NEXT: GRID FILL AT 25 GAMES (18 / 25)".

## Verification
`cargo test` passes with a test for the next-unlock text; the `records` shot shows the
totals.

## Notes
- From the 2026-11-05 brainstorm (D39).
