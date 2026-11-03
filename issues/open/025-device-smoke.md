# [025] Device smoke on a real iPhone and Android phone

**Status:** open
**Milestone:** M2
**Depends on:** [020], [022], [023]
**Created:** 2026-11-02
**Closed:** -

## Problem
Simulators and emulators hide GPU, touch and thermal behaviour. M2 is done only when the
game runs on hardware.

## Approach
Pedro installs the iOS build (`just ios-device`) and the Android build (`just android`) on
his phones and plays 5 minutes each: draw, claim, eat an enemy, catch the bomb, die,
restart, background and resume.

## Verification
Pedro reports both runs clean, with the device models, in Notes.

## Notes
- **owner-run**.
- [020] (2026-11-02): iPhone: `IOS_DEVICE=<id> just ios-device` (ids from `xcrun devicectl list devices`); Xcode signs with team `B8Y7SGQP5V` and may ask to register the device. Android: `just android-run` with `ANDROID_SERIAL` set to the phone.
- From [024] (2026-11-02), optional: run a build with the frame-time log (AGENTS.md
  "Frame-time log") and note the `slow update` lines and frame times. If ordinary frames
  (no claim) take over 16 ms on a phone, file a post-processing quality issue (HDR, bloom,
  SMAA, CRT): the emulator and the simulator cannot measure the GPU cost.
- From [036] (2026-11-03): on both phones, a tap on PLAY starts a run; a press on PLAY that
  slides off and lifts outside does nothing. The iOS simulator has no headless tap, so this
  is the iOS check of the title buttons.
- From [079] (2026-11-03): iPhone: the home indicator is hidden and a swipe up from the
  bottom first shows it (a second swipe goes home). Android with gesture navigation: quick
  swipes from the side edges draw lines and never leave the game; Back in play pauses.
