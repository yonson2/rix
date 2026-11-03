# [079] Keep OS edge gestures out of play

**Status:** done
**Milestone:** M2
**Depends on:** [020], [021], [023]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Pedro, 2026-11-03: Rix is edge-centric, the finger touches the screen edges a lot, and
that triggers the OS gestures (back, home, app switch). An accidental back swipe on
Android today calls `moveTaskToBack(true)` (`MainActivity.java`, [023]): the run leaves the
screen.

Today:
- iOS (`src/base.rs`): `preferred_screen_edges_deferring_system_gestures: ScreenEdge::All`
  and `prefers_status_bar_hidden: true`. The home indicator stays visible.
- Android (`MainActivity.java`): the deprecated `SYSTEM_UI_FLAG_IMMERSIVE_STICKY` flags; no
  gesture exclusion; Back moves the task to the background.

## Approach
- iOS: add `prefers_home_indicator_hidden: true`. With the deferred edges, a swipe from
  the bottom first shows the indicator; only a second swipe goes home.
- Android:
  - Replace the deprecated flags with `WindowInsetsController` (API 30, minSdk 31): hide
    `systemBars()`, behavior `BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE` (the modern sticky
    immersive mode).
  - `setSystemGestureExclusionRects` on the decor view: the whole window (the system uses
    only the parts in the side gesture areas), so a side swipe draws a line, not Back. The system
    takes at most 200 dp per edge into account, but the limit does not apply while the
    navigation bar is stickily hidden (`View` and `WindowInsets` reference), which is the
    mode above.
  - Back pauses the game in play (as the pause key, [073]) and only on the title moves the
    task to the background. Needs a call from Java into Rust (a JNI static or a key
    event that Bevy reads) or a key that GameActivity forwards.
  - Not done: exclusion only during play. The whole window stays excluded: the game has no
    other use for the side swipe, and Back on the title still works with the Back key
    (three-button navigation is not affected by the exclusion).
  - The bottom home and app-switch gestures cannot be excluded (gesture-nav guide); the
    immersive mode makes them need a first swipe that shows the bar.
- Game side (only if a device run shows it is still needed): a margin between the walls
  and the screen edges.

## Verification
Android emulator with gesture navigation
(`adb shell cmd overlay enable com.android.internal.systemui.navbar.gestural`): a swipe
from the left edge and one from the right edge during play
(three quick slow swipes per side, AGENTS.md "Android") keep Rix in front (`adb shell
dumpsys window | grep mCurrentFocus` names `top.cosas.rix`) and draw a line; Back in play pauses the
game; Back on the title moves the task to the background. iOS: `just ios-sim` builds and
starts; the home indicator check is on a device ([025]).

## Notes
- Sources: Android gesture-nav guide
  (https://developer.android.com/guide/navigation/gesturenav): "games can use
  `Window.setSystemGestureExclusionRects()` ... only exclude these areas when necessary,
  such as during gameplay"; "Apps can't opt out of these gestures [home/quick switch]".
  `View.setSystemGestureExclusionRects` reference: 200 dp limit on the vertical extent,
  not applied while the navigation bar is stickily hidden.
- Pedro has an upstream Bevy PR that surfaces the iOS gesture API.
- 2026-11-03, baseline on the emulator (old flags, gesture navigation): the first side
  swipe shows the bars and reaches the game (a wall-to-wall swipe from the screen edge
  claims 7 %); three quick swipes from the left edge fire Back, and Rix goes to the
  launcher. This is Pedro's problem.
- 2026-11-03, after the change: three quick swipes from each side keep Rix in front, the
  bars do not show, a wall-to-wall swipe from the right screen edge claims 7 %; Back
  (`input keyevent 4`) in play shows PAUSED, a second Back goes to the launcher; Back on
  the title goes to the launcher; no `UnsatisfiedLinkError` (the JNI symbol of the `rix`
  rlib is exported from `librix_mobile.so`).
- Back state (`src/game/pause.rs`, `back`): one `AtomicU8` (away, running, pressed) that
  `OnEnter`/`OnExit(IsPaused::Running)` set; the JNI call moves running to pressed with a
  compare and swap, so a run that ends at the same moment does not leave a stale press.
- `setDecorFitsSystemWindows` is deprecated in API 35 (javac note); it stays for API 31 to
  34 (minSdk 31).
