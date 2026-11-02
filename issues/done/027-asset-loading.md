# [027] Asset loading state - load every asset once

**Status:** done
**Milestone:** M3
**Depends on:** [026]
**Created:** 2026-11-02
**Closed:** 2026-11-02

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
- Done 2026-11-02: `src/asset_tracking.rs` (bevy_new_2d's, as is), `Screen::Loading`
  replaces `Screen::Setup` and moves to `Title` when every collection is in. Collections:
  `FontAssets` (`theme`) and `ItemAssets` (`basic_bomb`). The menu, HUD, game over, pause
  overlay and the item countdown (which loaded the font every frame) take the handle.
- The grep from the Verification line matches only the two collections' `FromWorld`
  impls (the loading code) and `crt.rs`, whose render pipeline loads its shader in the
  render world, as Bevy's post-process example does. Material2d shaders load by
  `ShaderRef` path inside Bevy.
- Shots: a variable load time would move the game start, so a shot run now pauses virtual
  time while loading and counts its script from the first title frame. Game, menu and seed
  7, 10 shots are the same in 3 of 3 runs each; they differ from the pre-[027] pictures by
  a few hundred pixels (CRT clock and enemy phase), and both still show the font. The
  flow census gains one `Loading` line and is otherwise the same.
- Pipeline warm-up (from [026]) moved to [040]: measure a first-kill hitch on a device
  before adding warm-up entities.
