# [073] Pause button and pause menu

**Status:** open
**Milestone:** M4
**Depends on:** [034], [037]
**Created:** 2026-11-02
**Closed:** -

## Problem
Since [034] the game pauses on suspend, focus loss and Escape, and a tap resumes. A phone
player has no way to pause on purpose, and the pause offers no restart or quit.

## Approach
A pause button in the HUD ([037] places it, for example in the top wall band) and a pause
menu: resume, restart, quit to the menu. A press on the button must not start a line:
`handle_input` skips presses inside the button's rect. Restart and quit leave `InGame`
from `Paused`; check that `OnExit(IsPaused::Paused)` restores the time.

## Verification
A scripted run presses the button: the game pauses and no line starts; "Restart" begins a
new run at level 1; "Quit" shows the menu; time runs again after each (test on the
states and `Time<Virtual>`).

## Notes
- Split from [034] (2026-11-02): the button and the menu need the HUD redesign ([037]).
- From [048] via [085] (2026-11-03): the menu also gets SETTINGS. The settings screen is
  `Screen::Settings`, so going there from a run would end it. Show the rows of
  `screens::settings` as a panel inside `IsPaused::Paused` (a sub-state or a flag), so the
  run stays. Decide whether a tap outside the buttons still resumes.
