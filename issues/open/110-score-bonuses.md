# [110] Score bonuses: overclaim and multi-eat

**Status:** open
**Milestone:** M7
**Depends on:** -
**Created:** 2026-11-05
**Closed:** -

## Problem
The points come from the claimed area, the kills and the items, so a score shows little
skill. The leaderboards ([068]) need a score that does. D39: the scoring may change, and it
changes before the boards go live, because a change after that makes the old scores
unfair.

## Approach
- **Overclaim:** the claim that clears the level pays a bonus for each whole percent above
  the target (75 %), with the level factor (`level_points`) and the Multiplier. A pop-up
  shows "+N OVER".
- **Multi-eat:** one claim that eats k enemies (k of 2 or more) pays k times
  `ENEMY_POINTS` for each of them, with a pop-up: DOUBLE, TRIPLE, QUAD, then "k x". A kill
  by a bomb or during the Star keeps its single pay.
- Pure functions with unit tests next to the scoring (`src/game/level.rs`). Tune the values
  in a dev run, then on Pedro's phone.

## Verification
`cargo test` passes with tests for both bonuses; `just replay` passes (update
`tools/replay-hashes.txt` in the same commit only if the level 1 points move on purpose);
`just shot` shows a pop-up.

## Notes
- From the 2026-11-05 brainstorm (D39). Not in scope: a speed bonus and a close call
  ([109] Notes).
