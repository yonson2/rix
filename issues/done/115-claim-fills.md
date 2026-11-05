# [115] Claimed-area fills

**Status:** done
**Milestone:** M8
**Depends on:** [078]
**Created:** 2026-11-05
**Closed:** 2026-11-05

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
- Done 2026-11-05. `FillSkin` (SOLID, SCANLINES, GRID, STARS) in `Skins.fill` (serde
  default: an old save keeps SOLID). `SkinAssets::fill` gives a `ColorMaterial` with a gray
  tile made in code (SCANLINES: 5-unit bands every 10; GRID: 2-unit lines every 16; STARS:
  12 stars in a 64-unit tile on a darker fill), one pixel per world unit, repeated. Its gray
  scales the claim's vertex color, so the flash and the level-clear fade work unchanged.
  `Shape` meshes now carry UVs (the positions), and `mesh_shapes` keeps a shape's own
  material. `handle_draw_polygon` gives each claim the chosen fill.
- Dead end: nearest sampling. A phone draws a world unit over 2 to 3 pixels, so the bands
  had uneven widths under the CRT pass; linear sampling fixed it. STARS at a dim of 110
  read too close to the free field; 150 reads as claimed.
- The fills have no unlock yet: `FillSkin::ALL` and `name` are behind
  `#[cfg(feature = "dev_tools")]` (only `RIX_SKINS` uses them); [114] removes the cfg.
- Evidence: `cargo test` 148 passed (fill tiles, a save without a fill, shape UVs); clippy
  clean with and without `dev_tools`; `just replay` unchanged; `just shot` md5
  e170997305a806320c6f9f967cf3cf5b (unchanged). Store-scene shots (seed 14, level 6, line
  set 2) at 585 x 1266 and 1179 x 2556 with each fill (`RIX_SKINS=<fill>`), looked at.
