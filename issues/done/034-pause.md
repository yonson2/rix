# [034] Pause

**Status:** done
**Milestone:** M3
**Depends on:** [026]
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
`IsPaused` exists and most systems run only in `IsPaused::Running`, but nothing ever enters
`IsPaused::Paused`. Physics keeps running regardless (avian is not paused). A phone call or
a swipe to another app kills the run.

## Approach
A pause button in the HUD and a pause menu (resume, restart, quit to menu); pause on app
suspend ([023]) and on window focus loss on desktop; pause physics time (`Time<Physics>`)
and virtual time together.

## Verification
In a `just shot`-style scripted run: pause mid-drag, wait 2 s, resume: enemy positions did
not change during the pause.

## Notes
- 2026-11-02, closed (pulled forward: [023] needs "resume paused"). `src/rix/pause.rs`:
  `pause_on_signal` (in `Running`) enters `Paused` on `AppLifecycle::WillSuspend` or
  `Suspended`, `WindowFocused { focused: false }` or Escape; `resume_on_input` (in `Paused`)
  goes back on a touch `Ended`, a left-click release or Escape (on the release, so the
  press does not start a line). `OnEnter(Paused)` pauses `Time<Virtual>` and
  `Time<Physics>` and spawns a "PAUSED / Tap to resume." overlay (`DespawnOnExit`);
  `OnExit` unpauses. Both switches clear `Messages<TouchInput>` and
  `Messages<MouseButtonInput>`, so stale pointer messages neither resume nor draw.
  `user_input::cancel_line` runs on `OnExit(Paused)`: a touch that ended while paused
  would otherwise leave `InputState` pressed and block every later touch.
- Split: the HUD pause button and the pause menu (resume, restart, quit) are [073] (M4,
  after the HUD redesign [037]).
- Evidence: test `pause::suspend_freezes_bodies_until_a_tap` (MinimalPlugins + avian, a
  dynamic body at 100 u/s: moves, then `WillSuspend` -> `Paused`, x unchanged over 120
  frames = 2 s, a touch `Ended` -> `Running`, moves again); test
  `user_input::resume_cancels_the_line_and_releases_the_pointer`. Android emulator: Home
  mid-run, return -> PAUSED, the ball still 3 s later, a tap resumes; a drag suspended
  mid-line, then resumed: the next swipe draws and claims. 33 tests, clippy clean.
- Test setup lesson: a headless app with avian needs `AssetPlugin`, `ScenePlugin`,
  `MeshPlugin` and `app.finish()` before `update()` (plugins insert resources in
  `finish`), plus `init_asset::<Font>()` for `Text2d` fonts.
