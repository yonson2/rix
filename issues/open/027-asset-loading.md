# [027] Asset loading state - load every asset once

**Status:** open
**Milestone:** M3
**Depends on:** [026]
**Created:** 2026-11-02
**Closed:** -

## Problem
`asset_server.load("fonts/main.ttf")` is called in the menu, the HUD, game over and every
frame in `display_countdown` (`items/mod.rs`); the bomb texture loads on each spawn. Two
TODOs say "load assets in load step". On a slow phone the first frame of each screen can
show missing text.

## Approach
A loading screen state that loads fonts, textures, shaders and audio into handle
resources (bevy_new_2d's asset tracking pattern, [026]) and moves on when all are loaded.
Systems take the handle resources, never paths.

## Verification
`grep -rn 'asset_server.load\|load_asset' src/` only matches the loading module; `just
shot` and `just shot-menu` still render text.

## Notes
- From [026] (2026-11-02): also warm the custom pipelines (`ExplosionMaterial`,
  `PixelExplosionMaterial`, `GlowMaterial`, the CRT pass) during load, so the first kill or
  bomb does not hitch on a phone (bevy_new_2d ranked item 2).
