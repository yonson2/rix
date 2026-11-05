# [121] Challenge rule effects

**Status:** done
**Milestone:** M9
**Depends on:** [120]
**Created:** 2026-11-05
**Closed:** 2026-11-05

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
- 2026-11-05 result: the effects read `Rules` ([120]). Kinds: `spawn_enemies` picks
  evenly from the chosen kinds (each from level 1) in place of `SPAWN_TABLE`. Count:
  `Rules::count` scales the level's count and rounds it (1 enemy at 1.5x: 2). Speed: the
  relative speed of `Time<Physics>` (`set_speed` on `OnEnter(Screen::Gameplay)`), so every
  enemy and bullet moves faster with one setting and no change in the enemy code; the
  timers (a square's jump, a shooter's fire, a splitter's turn) keep their time. Target
  and start level: `new_run` sets `LevelProgress.target` and `Run.level` with the RNG that
  level has after a clear (`seed + level`; level 1: the seed). `RIX_LEVEL` keeps its own
  override (store art). Items: `rules::items_on` on `spawn_items` and `roll_item`.
- Pay (`Rules::pay`, on `level_points`): in percent, one value after the other: speed
  100/125/150/200, count 100/125/150/200, target 100 (75 %) / 75 (60 %) / 125 (85 %) /
  150 (90 %), no items 125. Kinds, countdown and start level pay as the normal game (the
  start level already doubles the points per level). It saturates as the score does.
- A challenge that starts past level 1 raises `Save::best_level` (and opens level skins)
  until [123] keeps challenge runs out of the best level, as its Approach says.
- Evidence: `cargo test` 180 passed (`the_kinds_rule_names_kinds_in_enemy_order`,
  `the_count_rule_scales_and_rounds_the_count`,
  `the_speed_target_and_start_rules_read_their_values`,
  `the_speed_rule_runs_the_physics_clock_faster`,
  `harder_rules_pay_more_and_easier_ones_less`, `no_items_turns_the_items_off`,
  `a_challenge_run_starts_at_its_level_with_its_target`,
  `a_challenge_spawns_only_its_kinds_and_scales_their_count`, and the level points test);
  clippy clean both ways; `just replay` passes with the old hashes and two challenge seeds
  (`1C50-0000-001`: balls only, 2x count, 2x speed, `fb59d003cbe4d3bb`;
  `12G2-1000-0000-2`: shooters only from level 5, no items, 60 %, `0565e43f80da3d1d`), the
  same at 30, 60 and 120 fps; `just shot` md5 `e170997305a806320c6f9f967cf3cf5b` as
  before; `RIX_SEED=1553609930047489 just shot` shows two balls at level 1 (21 points: 7
  at 3x), and the shooter seed three shooters at level 5.
