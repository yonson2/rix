# [094] Bug: a kill's pixel explosion freezes during the level change

**Status:** done
**Milestone:** M4
**Depends on:** [038], [082]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Pedro (D27): "Sometimes, when you kill an enemy and advance to the next level at the same
time, there is like a pixel-ish yellow remnant left." Cause: `effects::update_explosion_time`
is in `PausableSystems`, which run only in `IsPaused::Running`, a sub-state of
`Phase::Playing`. In `Phase::LevelTransition` the shader time stops, so the orange pixels of
the kill (`PixelExplosionMaterial`) stay frozen until the next level starts.

## Approach
`update_explosion_time` leaves `PausableSystems`, as `animate_corpses` did. The pause stops
virtual time, so a paused explosion still holds still.

## Verification
A test: in `Phase::LevelTransition` the explosion materials' time goes on; `cargo test`.

## Notes
- Done 2026-11-03. `update_explosion_time` is in `AppSystems::Update` only. The fx sparks
  (`Burst`) and the corpses were not pausable already, so the explosion was the one frozen
  part.
- Evidence: test `a_kill_explosion_plays_on_during_the_level_change` failed before the fix
  (the material time stood still in `Phase::LevelTransition`) and passes after it.
