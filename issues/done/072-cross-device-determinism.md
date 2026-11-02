# [072] Same seed, same run on every device and frame rate

**Status:** done
**Milestone:** M3
**Depends on:** [019], [026], [028]
**Created:** 2026-11-02
**Closed:** 2026-11-03

## Problem
D10 Q5 wants a seed to give the same game on any device, and [067] shares seeds. Since
[019] the field is the same everywhere, but the run can still drift: gameplay systems in
`Update` step by the frame time (square teleports, arrow chase, timers that draw from
`LevelInfo.rng`), so a 120 Hz phone and a 60 Hz one diverge; avian's float results may
differ between CPUs (arm64 phones, x86 web hosts) without its `enhanced-determinism`
feature; system order is not fixed ([026]).

## Approach
Move gameplay that moves bodies or draws from the RNG into `FixedUpdate`; turn on avian's
`enhanced-determinism` (check its cost on a phone, [024]); after [026] fixes the system
order, add a headless replay test: a seed and a scripted input run at two frame rates
(and on two targets in CI if cheap) gives the same hash of enemy positions and claimed
area after N seconds.

## Verification
The replay test passes: seed 1 with the scripted line at 30 and 60 frames per second ends
with the same state hash.

## Notes
- Found in [019] (2026-11-02): the field and the input no longer depend on the screen;
  this issue covers the rest of "same seed, same game".
- From [026] (2026-11-02): `AppSystems` orders input before its readers, and shot runs
  now replay with an arrow (seeds 3, 7, 10). `Update` still has 101 ambiguous pairs. To
  list them: set `ScheduleBuildSettings { ambiguity_detection: LogLevel::Warn, .. }` on
  `Update` (`app.edit_schedule`) for one run; system names need `--features bevy/debug`
  (use a separate `CARGO_TARGET_DIR`, it rebuilds Bevy). Order the pairs that share
  `LevelInfo` (the RNG) first.
- Done (2026-11-03):
  - Gameplay moved to `FixedUpdate` (64 ticks per second): line input
    (`reset_input_state_flags`, `handle_input_events`, `handle_input`), line and claim
    (`handle_draw_line`, `handle_clear_lines`, `handle_build_polygon`,
    `handle_draw_polygon`), enemies (collision, die, ball chaos and growth, square
    teleport, collision and pulse, arrow chase and collision), items (collision,
    activate, despawn, bomb), `despawn_after`, points, progress, `handle_current_level`.
    `Update` keeps the systems that only draw (colour transitions, arrow stroke, item
    blink and countdown, wall glow, explosion time, HUD, level-transition fade).
  - `FixedUpdate` runs on `SingleThreadedExecutor`: one fixed system order, so the four
    `GameRng` users (claim via the kill observer, bomb, ball chaos, square teleport) and
    the other ambiguous pairs run in the same order every run. No hand-made pair order.
  - avian `enhanced-determinism` is on (`Cargo.toml`; `libm` 0.2.11 -> 0.2.16 in the
    lock, which parry's `simba` needs). Cost: a throwaway bench (15 bodies, 6,400 steps,
    dev profile, deps at opt 3) gave about 438 µs per update without and 441 µs with,
    inside the ±10 % noise. The phone cost check stays with [025] / [024]'s frame log.
  - Shot harness: `RIX_SHOT_FPS`; the line script (`drive_line`, `FixedPreUpdate`) sends one
    touch per game tick from tick 64 for 12 ticks; `log_state` (`FixedLast`) logs
    `RIX_STATE`, a hash of enemy positions and velocities, the free region, points, kills
    and the RNG state after 160 ticks. `just replay` compares seeds 1, 2, 5, 7 and 8 at 30,
    60 and 120 fps.
- Evidence: `just replay` passes (same hash per seed at 30, 60 and 120 fps; also 24 fps
  for seeds 2, 5, 7, 10 and 12 by hand), with and without `enhanced-determinism` (the same
  hashes). The test can fail: with the ball chaos back in `Update`, seeds 2, 5 and 8 give
  different hashes at 30 and 60 fps (seed 1 alone cannot show it: its square's one bounce
  falls on a frame boundary). `cargo test` 38 passed, clippy clean both ways, fmt clean.
  Shots changed only in timing (seed 1 `2b434c91`, 7 `5f295e4a`, 10 `01f39cbb`, menu
  unchanged `e520ba21`); `just shot-flow` census as in [030]; the seeded part is the same
  in 3 runs.
- Limits:
  - State changes (level clear, death) apply at the next frame, so a low frame rate
    can run one more tick before them. The next level reseeds the RNG, so its start
    replays.
  - Level 1 has one enemy; the replay does not cover several enemies in one tick.
  - Cross-target (arm64 vs x86, wasm) is not checked: the replay renders, so CI cannot
    run it. Filed as [075].
  - Bodies and the line tip now step at 64 Hz on any screen. Smooth 90/120 Hz motion is
    [076].
