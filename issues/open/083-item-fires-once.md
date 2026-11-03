# [083] An item fires once, even when many line segments touch it in one tick

**Status:** open
**Milestone:** M4
**Depends on:** [033]
**Created:** 2026-11-03
**Closed:** -

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
