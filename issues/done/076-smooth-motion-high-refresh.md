# [076] Smooth motion on 90 and 120 Hz screens

**Status:** done
**Milestone:** M4
**Depends on:** [072], [025]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Physics steps at 64 Hz (avian in `FixedPostUpdate`), and since [072] the gameplay and the
line input step with it. Nothing interpolates the drawn positions between steps, so on a
90 or 120 Hz phone the bodies and the line tip move in 64 Hz steps: some frames show no
motion, which can read as judder. Not yet seen on a device.

## Approach
Check on a 120 Hz phone ([025]). If it judders, turn on avian's transform interpolation
(`PhysicsInterpolationPlugin::interpolate_all()` or `TransformInterpolation` on enemies
and items); the state stays in fixed ticks, so `just replay` still holds. A higher fixed
rate is the other option (more physics cost).

## Verification
A 120 Hz device shows smooth enemy motion (owner check), and `just replay` passes.

## Notes
- From [072] (2026-11-03). The 64 Hz physics step is older than [072]; [072] adds the line
  input to it.
- Result (2026-11-03): done before the device check, because the step mismatch also hits
  60 Hz screens: at 60 fps with 64 Hz physics, about one frame in 16 runs two steps, so a
  body jumps twice as far there. `base::physics()` adds
  `PhysicsInterpolationPlugin::interpolate_all()`. bevy_transform_interpolation 0.6 puts
  each `Transform` back to the tick's end value in `FixedFirst`
  (`complete_translation_easing`), so `FixedUpdate` reads stepped values.
  - Test `bodies_move_evenly_at_60_fps_with_64_hz_physics`: a body at 300 units/s, 63
    frames at 60 fps. Plain physics: frame steps spread by more than 4 units (one-step and
    two-step frames); with interpolation, by less than 0.5.
  - Square teleport: the repeating timer wraps on the teleport tick, so its alpha was back
    near 1 there and the interpolated frames would show it between its two spots.
    `flicker_alpha` keeps it at 0 on that tick (test
    `a_square_is_invisible_on_the_tick_it_teleports`).
  - `just replay`: the same five hashes as before (game state unchanged). Flow census
    unchanged. `just shot` md5 839615a6d1b69a4c0f1cb3eeee7ffffe (the square draws 1 px
    behind its tick position).
  - Owner check (120 Hz phone shows smooth motion) moved to [025].
