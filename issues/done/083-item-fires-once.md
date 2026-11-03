# [083] An item fires once, even when many line segments touch it in one tick

**Status:** done
**Milestone:** M4
**Depends on:** [033]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Found in [040]'s emulator run (2026-11-03, `target/emu040.log` 02:57:29.58): one swipe
across the bomb gave 7 `ActivateItem` messages in one tick (one per line segment that
touched it). `activate_item` despawns with a deferred command, so `get_entity` succeeds for
all 7: the bomb fires 7 times and pays `ITEM_POINTS` 7 times. With one enemy left, all 7
bombs pick it; the kill counts once (the observer's despawn lands first), but
`on_despawn_enemy` pays `ENEMY_POINTS` 7 times. Score 1,050 instead of 150 in one tick.
A slow frame (the emulator, a hitch on a phone) packs many segments into one tick, so it
is not only an emulator effect. Two bombs caught in one claim can also pick the same
victim.

## Approach
`activate_item` handles each item entity once per run of the system (skip repeats).
`on_despawn_enemy` pays points only for an enemy it really removes (inside the `if let`),
so two triggers for one enemy pay once. The bomb picks its victim among the enemies no
other bomb picked in the same tick. Unit tests for each.

## Verification
`cargo test` passes with a test that sends 7 `ActivateItem` for one bomb in one tick and
gets one `ActivateBasicBomb` and one `ITEM_POINTS`, and a test that triggers
`DespawnEnemy` twice for one enemy and gets one kill and one `ENEMY_POINTS`;
`just replay` passes.

## Notes
- Result (2026-11-03): `activate_item` skips an item it already fired in the same run
  and pays `ITEM_POINTS` only for an item it removes (it also logs only those);
  `on_despawn_enemy` pays `ENEMY_POINTS` inside its `if let`; `basic_bomb::activate`
  draws its victim from the enemies no bomb took in that run (`rng.choice` on a `Vec`,
  the same single draw as before, so `just replay` gives the same hashes). Tests:
  `an_item_touched_by_many_segments_in_one_tick_fires_once`,
  `an_eaten_enemy_counts_and_pays_once_and_a_state_exit_counts_none`,
  `two_bombs_in_one_tick_take_two_enemies`. Seed 24 (the scripted line claims the bomb and
  a segment touches it the next tick): 100 + 50 points, one activation logged. 61 tests,
  clippy clean, `just replay` passes, `just shot` md5 unchanged.
