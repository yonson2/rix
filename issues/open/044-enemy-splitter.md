# [044] New enemy - Splitter

**Status:** open
**Milestone:** M4
**Depends on:** [032], [012]
**Created:** 2026-11-02
**Closed:** -

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
