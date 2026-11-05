# [118] Kill counts in the wiki

**Status:** open
**Milestone:** M8
**Depends on:** [071]
**Created:** 2026-11-05
**Closed:** -

## Problem
The wiki shows each enemy kind, but not how many of each the player has eaten. A count
gives the wiki a reason to come back to.

## Approach
- `Stats` counts the kills per kind, added at the end of a run as `kills` is.
- Each enemy page in the wiki shows "EATEN: 1,204", with a mark at 100 and at 1000.

## Verification
`cargo test` passes with a test for the count per kind; a `wiki` shot shows the count.

## Notes
- From the 2026-11-05 brainstorm (D39).
