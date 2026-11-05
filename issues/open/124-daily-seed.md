# [124] Daily seed

**Status:** open
**Milestone:** M9
**Depends on:** [109], [123]
**Created:** 2026-11-05
**Closed:** -

## Problem
A seed for each day gives every player the same field that day, and a reason to come back
(see [109] Q1). [012] Q4 said no daily seed for v1; D39 opens the question for 1.3.

## Approach
If [109] Q1 is yes:
- The day's code comes from the UTC date: the same for every player, with no server.
- It sits at the top of the CHALLENGES screen. Proposal: the day's seed with one of the
  curated rule sets, in turn (a daily challenge).
- `Save` keeps the best of the day and the days in a row.

## Verification
`cargo test` passes with tests for the date to code (the same all day, a new one the next
day) and the days-in-a-row rules; the `challenges` shot shows the daily entry.

## Notes
- From the 2026-11-05 brainstorm (D39); waits for [109] Q1.
