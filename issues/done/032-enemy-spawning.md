# [032] Data-driven enemy spawning with weights and tiers

**Status:** done
**Milestone:** M3
**Depends on:** [026], [028]
**Created:** 2026-11-02
**Closed:** 2026-11-03

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
- Done 2026-11-03:
  - `enemies/rng.rs`: `EnemyKind`, `SpawnRule { kind, levels, weight }` (a kind can have
    one row per level band), `SPAWN_TABLE` (all three kinds from level 1, weight 1: the
    old uniform pick) and `pick_kind`, a weighted pick that draws `rng.u64(0..total)`.
    On 64-bit that is the draw `Rng::choice` made (`usize` is `gen_u64` there), so the
    shots stay the same; on 32-bit (wasm) `usize` draws `gen_u32`, so `u64` keeps a seed
    the same on every target (noted in [075]).
  - `spawn_enemies` matches the kind and calls `world.run_system_cached(spawn_*)`;
    `spawn_items` does the same for the bomb. `EnemySystems`, `ItemSystems` and
    `setup_world` are gone (the `Screen::Setup` state went in [026]).
  - `generate_enemy_position` returns `Option<Vec2>` after at most 64 tries (was
    10,000, then the last spot). A spawn with no spot is skipped; a Square with no spot
    to teleport to dies through `DespawnEnemy` (a kill: explosion and points). The
    search stays in the field box: claimed shapes have colliders, so the spatial query
    already rejects claimed spots, and sampling the free region would change the draws.
  - Kept: the spawn functions stay systems (no bundle functions); they need
    `SpatialQuery`, meshes and the RNG, so a bundle function would only move those
    parameters.
- Verification 2026-11-03: `cargo test` 42 passed, among them
  `the_table_gives_these_kinds_at_levels_1_10_and_30` (seed 1: `[Square]`, 5 and 10
  kinds), `equal_weights_pick_as_choice_did`, `a_kind_spawns_only_in_its_band_and_by_weight`
  and `a_full_field_has_no_spot_and_a_square_that_cannot_teleport_dies`. Clippy clean
  with and without `dev_tools`; fmt clean. `just shot` md5 unchanged for seeds 1, 7, 10
  (2b434c91…, 5f295e4a…, 01f39cbb…) and the menu (e520ba21…). `just replay` passes.
  `just shot-flow`: every kind count and step time is the same as HEAD; the total is 4
  lower on the loading screen (the four spawn systems now register on first use) and 3
  lower once one enemy kind has spawned.
