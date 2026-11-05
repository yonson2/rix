# [116] Kill effects

**Status:** open
**Milestone:** M8
**Depends on:** [078]
**Created:** 2026-11-05
**Closed:** -

## Problem
An enemy's death has one look (the white pop and the shards). D39 picks kill effects as a
new kind of look for the unlocks ([114]).

## Approach
- Two more looks for a kill; the current one stays the default. Each is new (for example a
  shockwave ring, or a cartoon puff from `examples/cartoon_explosion`).
- Look only: no physics, and no draws from `GameRng`, so replays stay the same ([104]).
- `RIX_SKINS` takes the kill effect as a fourth value.

## Verification
`just shot` with each effect at a kill shows it; look at the PNGs. `just replay` passes
unchanged.

## Notes
- From the 2026-11-05 brainstorm (D39).
- 2026-11-05: the current kill already spawns the pixel explosion
  (`PixelExplosionMaterial` in `on_despawn_enemy`, `src/game/enemies/mod.rs`), so a pixel
  burst is not a new look. It draws its shader seed from `GameRng`; a new effect must not.
