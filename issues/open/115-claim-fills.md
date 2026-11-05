# [115] Claimed-area fills

**Status:** open
**Milestone:** M8
**Depends on:** [078]
**Created:** 2026-11-05
**Closed:** -

## Problem
The claimed area has one look, a solid fill. D39 picks fills as a new kind of look for the
unlocks ([114]).

## Approach
- Three looks for the claimed area (placeholders: SCANLINES, GRID, STARS); the solid fill
  stays the default. A pattern in world space over the claimed meshes, in the neon
  palette, readable under the CRT pass.
- Look only ([067]): no change to the territory, the colliders or `GameRng`.
- `RIX_SKINS` takes the fill as a third value.

## Verification
`just shot` with each fill (through `RIX_SKINS`) shows it; look at the PNGs. `just replay`
passes unchanged.

## Notes
- From the 2026-11-05 brainstorm (D39: Pedro left the kinds of looks to the agent).
