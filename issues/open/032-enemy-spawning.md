# [032] Data-driven enemy spawning with weights and tiers

**Status:** open
**Milestone:** M3
**Depends on:** [026], [028]
**Created:** 2026-11-02
**Closed:** -

## Problem
- Spawning goes through `EnemySystems` (a resource of `SystemId`s registered in the
  `Setup` state) and an exclusive `spawn_enemies(world)` that picks a kind uniformly.
  todo.md: "revisit enemy spawning: I want different weights and tiers".
- `generate_enemy_position` (`utils.rs`) tries up to 10,000 random spots with a spatial
  query each; the Square calls it on every teleport. todo.md: "redo loop of square, if you
  can't find a pos after X, kill it".

## Approach
An `EnemyKind` enum with a spawn table (weight per kind per level band, first level it
appears); spawn through one observer or command per kind; drop the `Setup` state. Bounded
position search (e.g. 64 tries over the free region from [029]); on failure the Square
dies (todo.md) and other kinds skip the spawn.

## Verification
A unit test of the spawn table (levels 1, 10, 30 give the expected kinds and counts for a
fixed seed); `just shot` unchanged for the same seed where the table keeps old weights.

## Notes
- todo.md: "sometime, revisit enemy spawning: I want different weights and tiers"; "redo
  loop of square, if you can't find a pos after X, kill it".
- From [026] (2026-11-02): spawn each kind through a bundle function
  (`fn ball(..) -> impl Bundle`, the bevy_new_2d pattern) and `commands.run_system_cached`
  or an observer; that removes `EnemySystems`, `ItemSystems` and the `Screen::Setup`
  state they need. Plugins are `pub(super) fn plugin`; gameplay systems go in
  `PausableSystems` and `AppSystems::Update`.
