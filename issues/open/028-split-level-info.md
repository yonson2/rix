# [028] Split LevelInfo into run, level and RNG state

**Status:** open
**Milestone:** M3
**Depends on:** [026]
**Created:** 2026-11-02
**Closed:** -

## Problem
`LevelInfo` (`setup.rs`) holds the run (seed, points, kill count, level number), the level
(target, current progress) and the RNG, and `Deref`s to the RNG, so `level_info.f32()`
reads as a level property. Any system that needs randomness takes `ResMut<LevelInfo>` and
triggers `resource_changed::<LevelInfo>`, which redraws the HUD on every chaos bounce. The
area sampler also draws from the gameplay RNG, so the number of claims changes later spawns
for the same seed.

## Approach
Separate resources: `Run` (seed, score, stats, level), `LevelProgress` (target, claimed),
`GameRng` (seeded per level from the run seed); an area computation that uses no RNG
([029]). `Points` keeps its own `Display` (tested in [007]).

## Verification
`cargo test` passes; the HUD update system runs only when score, level or progress
change (a test or a log count over 10 s of play).

## Notes
