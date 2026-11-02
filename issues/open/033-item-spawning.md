# [033] Items spawn during play; a caught item keeps its kind

**Status:** open
**Milestone:** M3
**Depends on:** [026]
**Created:** 2026-11-02
**Closed:** -

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
