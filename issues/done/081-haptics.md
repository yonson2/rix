# [081] Haptics on iOS and Android

**Status:** done
**Milestone:** M4
**Depends on:** [040], [020], [021]
**Created:** 2026-11-03
**Closed:** 2026-11-03

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
- Done 2026-11-03. `src/haptics.rs`: the `Sfx` event asks for a pulse (claim light, eat and
  bomb medium, bounce none), `OnEnter(IsPaused::Dying)` a heavy one; `Pending` keeps the
  strongest of a frame, so the kills of one claim buzz once (test). `pulse` runs on the main
  thread (`NonSendMarker`, UIKit needs it) and logs `haptic <kind>`.
  - Android: JNI (`jni` 0.21) calls the instance method `MainActivity.haptic(int)` on the
    activity from `bevy::android::ANDROID_APP` (`ndk-context` holds the Application, not the
    Activity). Java maps 0, 1, 2 to `KEYBOARD_TAP`, `VIRTUAL_KEY`, `LONG_PRESS` through
    `performHapticFeedback` on the UI thread: no VIBRATE permission, and it follows the
    user's touch-feedback setting.
  - iOS: `UIImpactFeedbackGenerator` (`objc2-ui-kit` 0.2.2, the version winit uses) with
    `initWithStyle:` (deprecated since iOS 17, still works, needs no view).
- Emulator (four scripted games): 7 claims, 1 kill and 4 deaths logged 7 `haptic Light`,
  1 `haptic Medium` and 4 `haptic Heavy`, each next to its sound; no JNI error. Clippy is
  clean for desktop, `aarch64-apple-ios-sim` and Android (`cargo ndk ... clippy -p rix -p
  rix_mobile`). The simulator has no headless tap ([036]), so the iOS pulse lines are not
  checked there; the iOS check is the feel on the phone, in [025].
