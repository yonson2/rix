# [079] Keep OS edge gestures out of play

**Status:** open
**Milestone:** M2
**Depends on:** [020], [021], [023]
**Created:** 2026-11-03
**Closed:** -

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
  - Replace the deprecated flags with `WindowInsetsControllerCompat`: hide
    `systemBars()`, behavior `BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE` (the modern sticky
    immersive mode), and lay out under the cutout.
  - `setSystemGestureExclusionRects` on the decor view: a strip on the left and the
    right edge over the full height, so a side swipe draws a line, not Back. The system
    takes at most 200 dp per edge into account, but the limit does not apply while the
    navigation bar is stickily hidden (`View` and `WindowInsets` reference), which is the
    mode above.
  - Back pauses the game in play (as the pause key, [073]) and only on the title moves the
    task to the background. Needs a call from Java into Rust (a JNI static or a key
    event that Bevy reads) or a key that GameActivity forwards.
  - The bottom home and app-switch gestures cannot be excluded (gesture-nav guide); the
    immersive mode makes them need a first swipe that shows the bar.
- Game side (only if a device run shows it is still needed): a margin between the walls
  and the screen edges.

## Verification
Android emulator with gesture navigation
(`adb shell cmd overlay enable com.android.internal.systemui.navbar.gestural`): a swipe
from the left edge and one from the right edge during play
(`adb shell input swipe 2 1200 400 1200`) keep Rix in front (`adb shell dumpsys window |
grep mCurrentFocus` names `top.cosas.rix`) and draw a line; Back in play pauses the
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
