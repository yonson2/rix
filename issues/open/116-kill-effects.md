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
- Two more looks for a kill; the current one stays the default. One is a pixel burst, from
  `examples/pixel_explosion`; the second is new (for example a shockwave ring).
- Look only: no physics, and no draws from `GameRng`, so replays stay the same ([104]).
- `RIX_SKINS` takes the kill effect as a fourth value.

## Verification
`just shot` with each effect at a kill shows it; look at the PNGs. `just replay` passes
unchanged.

## Notes
- From the 2026-11-05 brainstorm (D39).
