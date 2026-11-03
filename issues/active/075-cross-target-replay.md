# [075] Replay without the renderer, checked on two targets in CI

**Status:** active
**Milestone:** M4
**Depends on:** [072]
**Created:** 2026-11-03
**Closed:** -

## Problem
`just replay` ([072]) shows that a seed gives the same state at any frame rate on one
machine. Seed sharing ([067]) needs the same state on every target: iOS and Android
(arm64), the web build (wasm on x86 and arm hosts). avian's `enhanced-determinism` is on,
but no run compares two targets. The replay runs the full shot harness, which renders, so
CI (no GPU) cannot run it.

## Approach
A replay mode with no render plugins (or a headless app test with the game plugins and
stub assets) that logs `RIX_STATE`; a CI job runs it on `ubuntu-latest` (x86_64) and
`macos-latest` (arm64) and compares the hashes. Add wasm if it runs under a headless
runner cheaply.

## Verification
The CI job passes with the same `RIX_STATE` hash for seeds 1, 2, 5, 7 and 8 on both
runners.

## Notes
- From [072] (2026-11-03): cross-target was "if cheap" there; it is not, because the
  replay renders.
- From [032] (2026-11-03): fastrand's `usize` (so also `Rng::choice`) draws `gen_u64` on
  64-bit and `gen_u32` on 32-bit (wasm): the same seed draws differently on the web.
  `pick_kind` uses `u64`. The bomb's victim pick (`rng.choice(enemies.iter())` in
  `items/basic_bomb.rs`) still uses `choice`; check every `usize` and `choice` draw here.
- Work (2026-11-03): `RIX_SHOT_SCENE=replay` runs the game line with Bevy's renderer off
  (`WgpuSettings` with no backends, no winit): no GPU, no picture; `log_state` ends the
  run. `tools/replay.sh` (also `just replay`) runs seeds 1, 2, 5, 7 and 8 at 30, 60 and
  120 fps and compares each hash with `tools/replay-hashes.txt`. The CI job `replay` runs
  it on `ubuntu-latest` and `macos-latest` in the `ci` profile.
  - Local, macOS arm64: the renderer-free hashes equal the rendered ones, in the `dev`
    profile and in the `ci` profile (opt-level 0): 49 s and 2 min 17 s with the build. A
    wrong line in the hash file fails the script.
  - The bomb's victim pick used `choice` (a `usize` draw): now `pick`, a `u64` index,
    the same draw on 64-bit (test `a_pick_draws_as_choice_on_64_bit`). No other gameplay
    draw uses `usize`; the territory fuzz's `usize` is a test.
  - Wasm is not in CI: see the note in [058].
- CI run 41581357025 (v0.4.9): Linux x86_64 failed on seed 8 (`0e8306e52073c9af`, macOS
  arm64 `d90988d97e91ad2b`); seeds 1, 2, 5 and 7 matched, and each runner agreed with
  itself at every frame rate. Cause: gameplay called the `f32` methods `sin`, `cos` and
  `atan2` (ball spawn and chaos, the arrow's aim, enemy spawn spots, a polygon angle),
  which call the platform libm; Apple's and glibc's differ in the last bits. Fix: all
  float functions go through `bevy::math::ops`, which uses the portable `libm` (avian's
  `enhanced-determinism` turns on `bevy_math/libm`). `clippy.toml` forbids the `f32`
  methods (Bevy's own list). After the fix macOS gives `0e8306e52073c9af` for seed 8, the
  Linux value; the hash file has it. The shot md5 stays `839615a6…`.
