# [081] Haptics on iOS and Android

**Status:** open
**Milestone:** M4
**Depends on:** [040], [020], [021]
**Created:** 2026-11-03
**Closed:** -

## Problem
A phone game feels flat without touch feedback. [040] added the visual juice (claim
flash, shake, sparks); the phone does not buzz on a claim, a kill or a death.

## Approach
A small native bridge, as [079] did for Back: Android calls
`View.performHapticFeedback` (or `Vibrator` with `VibrationEffect`) through a static
Java method that Rust calls over JNI; iOS uses `UIImpactFeedbackGenerator` through
`objc2`. One Rust function `haptic(kind)` with kinds light (claim), medium (kill, bomb)
and heavy (death), hooked where `fx` triggers `Shake` and `Burst`. Desktop and web do
nothing. A setting to turn it off belongs to [048].

## Verification
`cargo test` passes; on the emulator and the simulator each claim, kill and death logs
one `haptic <kind>` line (`adb logcat`, `simctl log show`); Pedro feels them on both
phones (owner-run part, with [025]).

## Notes
- Split out of [040] (2026-11-03): its approach said "Haptics on mobile with [023]".
