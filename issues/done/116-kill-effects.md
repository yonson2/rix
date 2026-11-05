# [116] Kill effects

**Status:** done
**Milestone:** M8
**Depends on:** [078]
**Created:** 2026-11-05
**Closed:** 2026-11-05

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
- Done 2026-11-05. `KillSkin` (PIXEL, the current look and the default; SHOCKWAVE;
  FIREWORK) in `Skins.kill` (serde default). `skins::spawn_kill` spawns `KillPart` sprites,
  `animate_kill_parts` grows, fades and moves them in `Update`: SHOCKWAVE is two soft rings
  in the enemy's color (150 and 95 units across, the second 0.1 s later); FIREWORK is 16
  fast and 8 slow streaks, spaced evenly, in cyan, yellow, orange and green, with drag and
  gravity, 0.8 s. The kill still draws the pixel explosion's seed from `GameRng` with every
  skin (test `every_kill_skin_draws_the_same_from_game_rng`).
- The cartoon explosion is the bomb's look, so it is not a kill look; the sparks of
  `Burst::kill` stay with every skin.
- `RIX_SHOT_SCENE=kill`: every enemy dies at tick 48, before the game line (at a high
  `RIX_LEVEL` the line can die first), the shot 12 ticks later.
- Dead end: the rings at 2x glow of the enemy's color bloomed to white; the color as it is
  reads. A shape enemy (arrow, splitter, shooter) still gets white rings: its kill color
  comes from the shared shape material, [126].
- `FillSkin` and `KillSkin` `ALL`/`name` are behind `#[cfg(feature = "dev_tools")]` until
  [114].
- Evidence: `cargo test` 150 passed; clippy clean with and without `dev_tools`; `just
  replay` unchanged; `just shot` md5 e170997305a806320c6f9f967cf3cf5b (unchanged);
  `RIX_SHOT_SCENE=kill` shots (seed 14 level 6, seed 1) with each skin, looked at.
