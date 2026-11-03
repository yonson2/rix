# [040] Juice and FX pass

**Status:** open
**Milestone:** M4
**Depends on:** [018], [011]
**Created:** 2026-11-02
**Closed:** -

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
