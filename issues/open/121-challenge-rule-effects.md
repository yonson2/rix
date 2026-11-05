# [121] Challenge rule effects

**Status:** open
**Milestone:** M9
**Depends on:** [120]
**Created:** 2026-11-05
**Closed:** -

## Problem
The rules from the seed code ([120]) must change the game.

## Approach
- Enemy kinds: the chosen kinds spawn from level 1; the level bands of `SPAWN_TABLE` do
  not apply.
- Enemy speed: every enemy and every bullet. Enemy count: the count per level.
- Clear target: `LevelProgress.target`. Items: none.
- Start level: the `RIX_LEVEL` start in `src/dev_tools.rs` moves into the game.
- Points factor: on `level_points`.
- Gameplay systems stay in `FixedUpdate` and draw only from `GameRng` ([072]).
- `tools/replay.sh` gets challenge seeds, with their hashes in
  `tools/replay-hashes.txt`.

## Verification
`cargo test` passes with a test per rule; `just replay` passes with the challenge seeds;
`just shot` with a challenge seed shows its enemies.

## Notes
- From Pedro's message (2026-11-05, D39): "all enemies be of the same type", "faster mobs
  + higher % of points".
