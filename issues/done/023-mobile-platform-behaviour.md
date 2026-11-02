# [023] Mobile platform behaviour - safe areas, suspend/resume, frame pacing

**Status:** done
**Milestone:** M2
**Depends on:** [020], [021], [019], [034]
**Created:** 2026-11-02
**Closed:** 2026-11-02

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
- [020] (2026-11-02): iOS already defers system gestures at all screen edges and hides the status bar (`src/base.rs`). Frame pacing from `feat/ios` (`WinitSettings::mobile()`, 60 Hz reactive focused mode, `desired_maximum_frame_latency` 3) was not taken: decide it here.
- 2026-11-02, closed.
- Safe areas: winit 0.30.9 exposes no safe-area insets, so this is a geometry check of the
  9:16 field ([019]). iPhone 16 portrait (393 x 852 pt, insets 59 top, 34 bottom): the
  field is 393 x 699, bands of 77 pt top and bottom: the notch and the home indicator sit
  in the bands. Android 1080 x 2400 portrait: bands of 240 px, above the cutout and the
  hidden bars. Landscape phones and iPads: the field is full height, so the home indicator
  (21 pt) lies over the bottom wall (35 u = 17 pt on an iPhone 16) and about 4 pt of play
  area; the screen edges defer system gestures on iOS ([020]), so a swipe there reaches
  the game first. Accepted: no HUD or play-critical content under it.
- Lifecycle: pause on suspend is [034] (pulled forward). Android Back used to destroy the
  GameActivity under the running native loop: the process stayed, logged "after
  GameActivity was destroyed" thousands of times, and a relaunch showed only the splash.
  `MainActivity` now registers an `OnBackPressedCallback` that calls `moveTaskToBack(true)`,
  so Back acts like Home (and pauses). Surface loss on Android: Bevy recreates it on
  resume (`onSurfaceCreated_native`), no app code needed. Audio: none yet; [041] note.
- Frame pacing: kept Bevy's default `WinitSettings::game()` (continuous when focused).
  The CRT pass and the menu animate every frame, so a reactive mode would stall them;
  frame cost belongs to [024]. `feat/ios`'s `desired_maximum_frame_latency` 3 not taken.
- Evidence, Android emulator (`target/shots/023-android-lifecycle.png`, local): in game ->
  Home -> back: PAUSED; 3 s later the ball has not moved; a tap resumes; Back -> relaunch:
  the same PID, PAUSED; no FATAL, panic or destroyed-activity line in logcat. Suspended
  mid-drag, resumed: the next swipe draws and claims. Rotation mid-run: [019]. iOS
  simulator: Settings brought to the front, then Rix again: the same PID, it draws.
- Not checked: iOS mid-run pause (`simctl` cannot tap to start a run) -> [025] device
  smoke. Seen: a swipe faster than ~0.5 s does not draw on the software-rendered emulator
  (the whole swipe falls into one or two frames); at 60 fps it does not happen.
