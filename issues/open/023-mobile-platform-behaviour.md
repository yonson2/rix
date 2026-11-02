# [023] Mobile platform behaviour - safe areas, suspend/resume, frame pacing

**Status:** open
**Milestone:** M2
**Depends on:** [020], [021], [019]
**Created:** 2026-11-02
**Closed:** -

## Problem
A phone game must avoid the notch, the dynamic island and the home indicator, survive the
app going to the background (Android drops the GPU surface on suspend), and not drain the
battery on the menu.

## Approach
- Safe areas: keep the playfield and the HUD inside the safe-area insets (check what Bevy
  0.20 or winit exposes; else a per-platform query).
- Lifecycle: on suspend, pause the game (needs [034]) and stop audio; on resume, recreate
  the surface (Bevy's mobile example handles `AppLifecycle`).
- Frame pacing: `WinitSettings` per state (reactive on menus, continuous in game);
  `feat/ios` set `desired_maximum_frame_latency` 3 and a 60 Hz reactive focused mode.
- Gestures: defer system gestures at screen edges on iOS (D6); keep Android back gesture
  from quitting mid-run.

## Verification
On the iOS simulator and the Android emulator: nothing important under the notch or the
home indicator; background then foreground mid-run resumes paused, with no crash.

## Notes
- [010] Q4 (2026-11-02): portrait and landscape, so rotation mid-run must keep the fixed
  playfield ([019]) and the touch mapping right.
- [022] (2026-11-02): on the emulator, Android shows its "Viewing full screen" hint over the top of the game on first start (immersive mode from `MainActivity`). System UI, once per install; check that the first tap still reaches the game.
