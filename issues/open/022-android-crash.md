# [022] Android crash at start - reproduce on the emulator and fix

**Status:** open
**Milestone:** M2
**Depends on:** [021]
**Created:** 2026-11-02
**Closed:** -

## Problem
Pedro: the game "segfaulted weirdly" on Android (Bevy 0.15.3 era, `feat/android`). iOS ran.
No logs were kept.

## Approach
Reproduce first on the Bevy 0.20 build from [021], then bisect by cause. Capture
`adb logcat` and the tombstone (`adb bugreport` or `/data/tombstones`), symbolize with
`ndk-stack` against the unstripped `.so`. Candidate causes to rule in or out, cheapest
first:
1. `androidx.games:games-activity` (Java side, 2.0.2 on the branch) not matching the
   GameActivity C glue that the `android-activity` crate bundles (0.6.0 on the branch). A
   mismatch crashes natively at start.
2. The emulator's Vulkan path (gfxstream on Apple Silicon) crashing in wgpu: try
   `-gpu swiftshader_indirect`, the GL backend, and a real device.
3. Render features: HDR `Rgba16Float` target, `Bloom`, `Smaa` and the custom CRT pipeline
   (hardcoded `TextureFormat::Rgba16Float` in `src/crt.rs`) on a GPU without them: start
   with a plain camera, add one feature at a time.
4. `libc++_shared.so` packaging (the CMake dummy hack) and the audio backend (`cpal` via
   `oboe`).
5. A debug build (the committed `.so` was debug): try release.

## Verification
Rix starts and plays on the `Medium_Phone_API_36` emulator for 2 minutes (draw, claim,
die, restart) with no crash in logcat; the cause and the fix are written in Notes.

## Notes
- Device check in [025].
