# [076] Smooth motion on 90 and 120 Hz screens

**Status:** open
**Milestone:** M4
**Depends on:** [072], [025]
**Created:** 2026-11-03
**Closed:** -

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
