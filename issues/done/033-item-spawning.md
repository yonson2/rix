# [033] Items spawn during play; a caught item keeps its kind

**Status:** done
**Milestone:** M3
**Depends on:** [026]
**Created:** 2026-11-02
**Closed:** 2026-11-03

## Problem
- Items spawn only on entering a level, with a 50% chance (`spawn_items`, TODO "this is
  bad, it should be at any point in time while playing"). todo.md: "redo item spawn".
- Catching an item inside a claim sends `ActivateItem(entity, Item::BasicBomb)` whatever
  the item is (`handle_build_polygon`); fine with one item, wrong with [046].
- The bomb spawns with `Mass(0.)` on a dynamic body; `generate_item_position` is unused.

## Approach
A timed spawner during `InGame` (interval and chance per level, from the seeded RNG); pass
the item's own `Item` kind on capture; give items a proper body or make them sensors.

## Verification
A test or a seeded `just shot` run shows an item spawned after the level started; a
unit test checks that capture forwards the item kind.

## Notes
- todo.md: "redo item spawn".
- Done 2026-11-03:
  - `items`: `ItemTimer` (repeating, `ITEM_INTERVAL` = 10 s, reset on
    `OnEnter(Phase::Playing)`) and `roll_item` (`FixedUpdate`, `TickTimers`): each time
    it finishes, `rng.f32() < ITEM_CHANCE` (0.5) spawns a bomb through
    `commands.run_system_cached(spawn_basic_bomb)`. The coin flip at level start stays,
    so the draws of the first 10 s are as before: shots, replay and census unchanged.
    One value for every level; [047] tunes interval and chance per level.
  - `handle_build_polygon` reads each item's `Item` and sends it in `ActivateItem`.
  - The bomb lost `Mass(0.)` in [065]; it is a dynamic body with a collider, pushed by
    enemies as before. `generate_item_position` stays in [035] (dead code).
- Verification 2026-11-03: `cargo test` 46 passed, among them
  `a_bomb_spawns_during_play_when_a_roll_succeeds` (no item at 9.5 s, one after the
  first roll) and `a_claim_activates_the_items_inside_with_their_own_kind`. Clippy clean
  with and without `dev_tools`; fmt clean. `just shot` md5 unchanged for seeds 1, 7, 10;
  `just replay` passes; `just shot-flow` census unchanged.
