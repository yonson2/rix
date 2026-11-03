# [040] Juice and FX pass

**Status:** done
**Milestone:** M4
**Depends on:** [018], [011]
**Created:** 2026-11-02
**Closed:** 2026-11-03

## Problem
The game has the pieces (bloom, CRT pass, explosion shaders, colour sparks) but claims and
hits feel flat. todo.md: "item outline". `feat/item_outline_shader` holds an outline shader
(`assets/shaders/outline_material.wgsl`, `examples/sprite_outline.rs`) on Bevy 0.15.

## Approach
Capture flash and sweep, small screen shake on death and bomb, particles on bounces, a
glowing trail on the drawn line, item outline (port the branch's shader), CRT parameters
tuned per device class. Haptics on mobile with [023].

## Verification
Before/after screenshots or short captures in Notes; frame time on the emulator stays
within [024]'s budget.

## Notes
- todo.md: "item outline".
- [011] Q1 (2026-11-02): keep the neon look with bloom and CRT; references Geometry Wars
  (particles, grid warp, bloom) and Hyper Light Drifter (colour and motion). D11: delete
  `feat/item_outline_shader` once its outline shader is ported here.
- From [026]/[027] (2026-11-02): bevy_new_2d warns that custom pipelines (CRT,
  `PixelExplosionMaterial`, `ExplosionMaterial`, `GlowMaterial`) compile on first use,
  so the first kill or bomb may hitch on a phone. Measure it on a device ([025]) first;
  if it hitches, spawn one hidden-but-drawn entity per material on `Screen::Loading`.
- From [038] (2026-11-03): the level clear has a "LEVEL N" banner (`level.rs`, `Banner`);
  the claimed shapes still only fade with a drift. A sweep or burst of the claimed area on
  a level clear belongs here.
- From [039] (2026-11-03): a kill shows a `Corpse` (white pop, shrink, 0.4 s) and the
  pixel explosion, which has no colour input. A burst in the enemy's own colour and
  per-kind death variants belong here.
- Result (2026-11-03): new module `src/game/fx.rs` (draws only, `Update`), plus a
  `shake` field in the CRT pass and an outline in `GlowMaterial`.
  - Claim flash: a new piece starts sRGB white and eases to cyan in 0.3 s (`Flash`).
    First try was `NEON_WHITE` (HDR 2.0): a claim of most of the field (seed 24) flooded
    the bloom and the screen went white, and the level clear's fade froze it at white.
    Now the peak is plain white and the flash keeps the fade's alpha.
  - Screen shake: `Shake` event adds trauma (death 0.8, bomb 0.6), offset `8 * trauma²`
    world units, real time. It moves the picture in the CRT pass (`PostProcessSettings::
    shake`), not the camera, so input, picking and the shot scripts never see it. Flow
    shot at 3.55 s (death at 3.5 s): the field edge moves 4 px, then returns.
  - Sparks: `Burst` event; a ball bounce throws 4 slow sparks in a half circle along its
    new velocity (first try threw them into the wall), a kill 16 fast ones in the enemy's
    colour times 3 (HDR, the plain green was too dim). Own `FxRng`, never `GameRng`:
    `just replay` passes and the flow census only gains the two observer entities.
  - Item outline: ported from `feat/item_outline_shader` into `glow_material.wesl`
    (8 taps at 3 to 4.5 texels, pulse on `globals.time`, fades with the blink intensity);
    the bomb uses `NEON_YELLOW`. The `glow_material` example sets no outline.
  - Shots: `target/shots/f040b_sheet.png` (seeds 24, 2, 5, 7, 8 at 1.22 s and 1.40 s:
    flash, settled claim, bounce sparks on seed 8, bomb outline), `f040c_sheet.png`
    (kills on seeds 52 and 61), `f040_die_sheet.png` (death shake). Default `just shot`
    md5 58e20249dec4137f0335322f42639bbb (1 pixel off by 1 from the old one, shader
    float noise), menu d8b068252156126286081f5955a68b80 (2 pixels off by 1).
  - Frame time, Android emulator (dev build with `rix/dev_tools`, the same 5 swipes: a
    bomb, kills, level 2, a death; logs `target/emu040.log` and `target/base040.log`):
    updates over 16 ms with [040] median 24.6 ms, p90 48.4, claim frames 69 and 110 ms;
    the same script on HEAD without [040] median 25.0, p90 48.7, claim frames 64 and 68,
    max 128 ms. Same spread: llvmpipe noise, no cost from the juice. The run also found
    [083] (one swipe fired the bomb 7 times).
  - Split out: haptics to [081]; level-clear sweep, line trail, per-kind deaths, CRT per
    device and shader warm-up to [082]. Shake and haptics toggles noted in [048].
