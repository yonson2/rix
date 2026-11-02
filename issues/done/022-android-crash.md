# [022] Android crash at start - reproduce on the emulator and fix

**Status:** done
**Milestone:** M2
**Depends on:** [021]
**Created:** 2026-11-02
**Closed:** 2026-11-02

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
- 2026-11-02, first run of the [021] APK on `Medium_Phone_API_36` (emulator, `-gpu
  swiftshader_indirect`, adapter "llvmpipe"): SIGSEGV 0.1 s after the adapter info, in
  `/vendor/lib64/hw/vulkan.ranchu.so (vk_common_SetDebugUtilsObjectNameEXT+256)`, thread
  `android_main`. wgpu only calls that with debug labels on (`InstanceFlags::DEBUG`, the
  default in debug builds). Logcat in the scratchpad (`logcat1.txt`).
- `feat/android` (deleted in [021], tip `7b6c0d1`) changed `base.rs` for Android:
  `WindowMode::BorderlessFullscreen(MonitorSelection::Primary)`, pinch/rotation/double-tap
  gestures on, and a `RenderPlugin` with `WgpuSettings { limits: WgpuLimits::default() }`
  (maybe for the old crash).
- 2026-11-02, closed. **Cause:** with `InstanceFlags::DEBUG` (wgpu default in debug
  builds) wgpu-hal 30 enables `VK_EXT_debug_utils` and names every Vulkan object; the
  emulator driver (`vulkan.ranchu.so`, llvmpipe) segfaults in
  `vk_common_SetDebugUtilsObjectNameEXT`. Release builds never set `DEBUG`. **Fix:**
  `src/base.rs` removes `InstanceFlags::DEBUG` on `target_os = "android"` (a
  `RenderPlugin` with `WgpuSettings`); only debug labels are lost.
- Evidence: debug APK on `Medium_Phone_API_36` (`-gpu swiftshader_indirect`): the menu draws
  with CRT and bloom; a 125 s `adb shell input` loop (tap, swipe wall to wall, 20 rounds)
  claimed area, reached level 3, died twice and restarted, same PID throughout, no
  `FATAL`/`SIGSEGV`/panic in logcat. `cargo ndk ... clippy --package rix_mobile -- -D
  warnings` is clean.
- Seen, already tracked: the field stretches to the tall screen ([019]); "Enemies Killed"
  counts the enemy that killed you ([030]); the Android immersive-mode hint covers the top
  on first start (system UI, [023]).
- Dead end: `adb install -r` of the 137 MB debug APK failed with
  `INSTALL_FAILED_INSUFFICIENT_STORAGE` (780 MB free on the emulator); `adb uninstall
  top.cosas.rix` first works. The Gradle feat/android hacks (libc++ CMake dummy, limits)
  are not needed.
