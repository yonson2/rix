# [110] Score bonuses: overclaim and multi-eat

**Status:** done
**Milestone:** M7
**Depends on:** -
**Created:** 2026-11-05
**Closed:** 2026-11-05

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
`cargo test` passes with the tests for both bonuses, their pay with the level and the gem,
the pop-up labels and the multi-eat pay of a claim; `just replay` passes with the hashes in
`tools/replay-hashes.txt` (the scripted line clears nothing and eats nothing).

## Notes
- From the 2026-11-05 brainstorm (D39). Not in scope: a speed bonus and a close call
  ([109] Notes).
- Values: `OVER_POINTS` = 10 per whole percent over 75 % (a clear at 88 % pays 130 at
  level 1, more than the claim's own 1 point per percent); a claim that eats k enemies
  pays k x 50 for each (2: 200, 3: 450, 4: 800 at level 1). Both double per level and
  with the gem. Tune them on the phone.
- Pop-ups: "+N OVER" at the field's centre; DOUBLE!, TRIPLE!, QUAD!, then "5X!" at the
  eaten enemies' centre, kept 110 units inside the field's sides. The look was checked once
  with a temporary trigger at the scripted claim (two pop-ups, readable, in the field).
- Result (2026-11-05): 138 tests pass, clippy clean with and without `dev_tools`, fmt
  clean; `just replay` same hashes; `just shot` md5 e170997305a806320c6f9f967cf3cf5b (as
  before).
