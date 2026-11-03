# [044] New enemy - Splitter

**Status:** done
**Milestone:** M4
**Depends on:** [032], [012]
**Created:** 2026-11-02
**Closed:** 2026-11-03

## Problem
Three enemy kinds (Ball, Square, Arrow) give little variety. todo.md: "The Splitter - When
hit or after a set time, splits into two smaller, faster versions of itself."

## Approach
Pedro's spec `docs/design/splitter.md` sketches one design: a "Queen" octagon that, when claimed,
spawns two faster "Soldiers"; straight-line movement with random turns. Confirm the design
in [012], then build it on the spawn table ([032]).

## Verification
A seeded run spawns a Splitter; claiming it spawns two soldiers (test or scripted shot).

## Notes
- todo.md: Splitter line above. `docs/design/splitter.md` (in git since D15) holds a step
  list written in April 2025 for Bevy 0.15 and the old code (`Mesh2dBundle`, events).
  Adapt it to Bevy 0.20 (`Mesh2d`, messages) and the spawn table ([032]);
  `generate_enemy_position` is in `utils.rs` now; its step 8 (kills counted twice) is [030].
- [012] Q1 (2026-11-02): the Splitter is in v1. `docs/design/splitter.md` is the
  starting design.
- From [026] (2026-11-02): `src/rix/` is now `src/game/` (the design doc's paths are
  updated). Plugins are functions now: write `pub(super) fn plugin(app: &mut App)` in
  `splitter.rs` and add `splitter::plugin` in `enemies::plugin`, not a `SplitterPlugin`.
  Gameplay systems go in `PausableSystems` and `AppSystems::Update`.
- Done 2026-11-03. `src/game/enemies/splitter.rs` after `docs/design/splitter.md` (its
  header lists the changes). `EnemyKind::Splitter` from level 2, so level 1 keeps its
  draws: the `just replay` hashes and the `just shot` image are unchanged. Kills, points,
  corpse, burst, sound and haptic come from `on_despawn_enemy` as for any enemy.
- Verified: tests `only_a_dead_queen_hatches_soldiers` (a Soldier kill hatches nothing),
  `a_hatch_turns_into_two_soldiers_at_free_spots` (with avian's spatial query) and
  `the_queen_outline_matches_her_collider`; the spawn-table test pins the new picks at
  levels 10 and 30. A temporary level-1 patch (not committed) drew a Queen and a Soldier
  with seed 3 (`target/shots/s044-*.png`): purple octagon and ellipse, neon edge.
