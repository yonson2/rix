# [028] Split LevelInfo into run, level and RNG state

**Status:** done
**Milestone:** M3
**Depends on:** [026]
**Created:** 2026-11-02
**Closed:** 2026-11-02

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
- From [024] (2026-11-02): the area measure (`claimed_fraction`) no longer draws from
  the RNG, so claims no longer change later spawns for a seed.
- Done (2026-11-02): `LevelInfo` became `Run` (seed, level, points, kill count),
  `LevelProgress` (target, current) and `GameRng` (`src/game/setup.rs`); `new_run`
  inserts the three for a seed (game over, `RIX_SEED`). Spawners, chaos, bomb and the
  explosion seed take `ResMut<GameRng>`; nothing that only draws a random number changes
  `Run` or `LevelProgress`. The HUD runs on `hud_changed` (`Run` or `LevelProgress`
  changed); test `hud_updates_only_when_run_or_progress_change` counts 1 HUD run in 10
  frames of RNG draws, then 1 more after a progress change.
- Evidence: `cargo test` 39 passed; clippy clean with and without `dev_tools`; fmt
  clean. Shots identical to [027] (md5, 3 runs each): game seed 1 `109eb09e`, menu
  `e520ba21`, seed 7 `480e261e`, seed 10 `ecf6bd76`. `just shot-flow` census identical to
  [027] in 2 runs.
- The double kill count stays for [030].
