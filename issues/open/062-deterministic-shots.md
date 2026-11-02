# [062] Dev harness: fixed frame time so seeded shots are deterministic

**Status:** open
**Milestone:** M0
**Depends on:** [006]
**Created:** 2026-11-02
**Closed:** -

## Problem
`RIX_SEED` fixes the random numbers but not time: the scripted line and the screenshot run
on real seconds, and physics steps follow the real frame rate. With seed 1 (one square)
two runs match (PSNR 62.6 dB), but with seed 4 (an arrow at level 1, it chases the finger)
the same build dies in some runs and claims the strip in others (found in [013]: 8 of 10
runs died on the new build, 7 of 10 on the old one). Upgrade steps ([014]-[018]) compare
shots between builds, so a run must not depend on the machine's frame timing.

## Approach
When `RIX_SHOT` is set, `DevToolsPlugin` inserts
`TimeUpdateStrategy::ManualDuration(1/60 s)`: every frame advances virtual time by the
same step, so the gesture, the fixed-step physics and the screenshot happen at the same
simulated moment on every run. Add a `just shot` variant or env var for the arrow seed if
it helps the upgrade checks.

## Verification
Ten runs of `RIX_SEED=4 RIX_SHOT=... cargo run --features dev_tools` give the same outcome,
and any two of their PNGs differ by less than 0.01 % of pixels (`magick compare -metric AE
-fuzz 3%`).

## Notes
- Seeds with an arrow at level 1 (0.15 build): 4, 7, 10, 13. Run the binary directly only
  with `BEVY_ASSET_ROOT=<repo>`, or assets (font, shaders) do not load.
