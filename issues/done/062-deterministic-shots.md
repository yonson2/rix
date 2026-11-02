# [062] Dev harness: fixed frame time so seeded shots are deterministic

**Status:** done
**Milestone:** M0
**Depends on:** [006]
**Created:** 2026-11-02
**Closed:** 2026-11-02

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
- Done. `RIX_SHOT` runs: `TimeUpdateStrategy::ManualDuration(1/60 s)`; the gesture runs in
  `PreUpdate`, so every `Update` reader sees a touch in the frame it is sent; the gesture
  and the give-up timer count `Time<Real>` (also fixed per frame); game time pauses when
  the screenshot is requested, because the capture can land a frame or two later (exit
  frame varied between f183 and f185 in traces).
- Main cause found with per-frame traces: `spawn_items` and `spawn_enemies` both ran on
  `OnEnter(InGame)` with no order, and `spawn_items` draws a coin flip from the level RNG
  first or not, so the same seed picked a different enemy (seed 4: arrow in some runs,
  ball in others). Fix in game code: `spawn_items.after(spawn_enemies)`. The enemy that a
  seed gives at level 1 changed with this (arrow seeds now: 7, 10).
- A single-threaded executor did not help (the order of unordered systems still changed
  between processes), so it is not used.
- Result: seed 4, 10 runs: 0 px differ. Seed 1 (`just shot`), 10 runs: at most 14 px, and
  it matches the [013] shot (0.6 px). Seeds 7 and 10 (arrow) still split into two outcomes
  (up to ~3,200 px): `point_arrow_to_cursor` and `handle_input` have no order in `Update`,
  so the arrow sometimes reacts a frame later. Bevy's ambiguity detector lists dozens of
  such pairs in `Update` (most on `LevelInfo` and events); ordering them is [026] (Notes).
  Use seed 1 for before/after comparisons until then.
- macOS: Pedro asked that harness runs stop taking the focus (2026-11-02). winit activates
  the app at launch and Bevy 0.15 cannot turn that off, so `give_focus_back` (exclusive
  system, main thread) re-activates the app that was in front at start (`NSWorkspace`
  `frontmostApplication` read in `Plugin::build`) and hides rix. Measured: rix in front for
  about 0.5 s, then the previous app again; the hidden window still renders the shot.
  `NSApplication::deactivate` alone did nothing; `hide` alone gave the focus to another app.
  `objc2-app-kit`/`objc2-foundation` are normal macOS dependencies (winit already builds
  them) so all builds share one feature set. Bevy 0.16+ may expose winit's
  `with_activation_policy`; check at [014]-[018].
