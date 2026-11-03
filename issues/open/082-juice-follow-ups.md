# [082] Juice follow-ups: level-clear sweep, line trail, CRT per device, shader warm-up

**Status:** open
**Milestone:** M4
**Depends on:** [040], [025]
**Created:** 2026-11-03
**Closed:** -

## Problem
[040] shipped the claim flash, the screen shake, the sparks and the item outline. Its
other ideas are still open: the level clear only fades the claimed shapes with a drift;
the drawn line has no trail; the CRT pass uses one set of parameters on every screen; the
custom pipelines (CRT, explosions, glow) compile on first use, which may hitch on a phone.

## Approach
- Level clear: a sweep of light across the claimed area or a burst of sparks along its
  edge (`fx::Burst`), before the "LEVEL N" banner.
- Line: a few sparks or a glow at the line's head while the finger draws.
- Per-kind death variants (a square shatters, an arrow spins away), in the enemy colour.
- CRT and shader warm-up wait on [025]'s device numbers: tune `PostProcessSettings` per
  device class if the scanlines or the grid look wrong on a phone; if the first kill or
  bomb hitches, spawn one hidden-but-drawn entity per material on `Screen::Loading`.

## Verification
`cargo test` and `just replay` pass; shots of a level clear and a line in progress in
Notes; frame time on the emulator stays within [024]'s budget.

## Notes
- Split out of [040] (2026-11-03).
