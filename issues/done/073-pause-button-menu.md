# [073] Pause button and pause menu

**Status:** done
**Milestone:** M4
**Depends on:** [034], [037]
**Created:** 2026-11-02
**Closed:** 2026-11-03

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

## Result (2026-11-03)
- HUD: a pause button ("II", 52 x 40, right of the level badge; the row's middle stays
  free). It pauses only in `IsPaused::Running`. `handle_input_events` drops a press on any
  `NeonButton`, so the press, its moves and its release draw no line.
- Pause menu (`PausePanel::Menu`, a sub-state of `IsPaused::Paused`): "PAUSED", the seed
  code ([067]), RESUME, RESTART, SETTINGS and MENU. The dim is 85 % (60 % let the neon
  enemies glow through the text).
  - RESTART starts a new run (random seed, as RETRY) through `Screen::Restart`, a
    one-frame screen: setting `Screen::Gameplay` again runs its exit and enter schedules
    but keeps `Phase` and `IsPaused`, so the run stayed paused (the test found it).
  - SETTINGS shows the settings panel (`PausePanel::Settings`) over the run;
    `settings::spawn_panel` now builds it for both places; BACK returns to the menu.
  - MENU starts a fresh run and shows the title, as the game over's MENU does.
  - A run left by RESTART or MENU is not recorded in the stats or the best score.
- Decision (mine, Pedro can change it): a tap outside the buttons does not resume; RESUME
  and Escape do. A stray tap must not drop the player back into a run.

## Evidence
- Tests: `the_menu_shows_settings_restarts_and_quits_with_time_running` (SETTINGS, a
  setting row steps the save, BACK, RESTART gives level 1 in `IsPaused::Running`, MENU
  gives the title; `Time<Virtual>` and `Time<Physics>` run after each),
  `suspend_freezes_bodies_until_resume` (a tap elsewhere keeps the pause),
  `only_a_touch_start_or_a_left_press_is_a_press`. 93 tests pass; clippy, fmt.
- A shot run patched to tap the HUD button and then SETTINGS (touches through both input
  streams): the run paused, `UserPath.is_drawing` stayed false, then the panel was
  `Settings`. Shots `target/shots/s073-menu.png`, `s073-settings.png`, `s073-hud.png`,
  and `s073-title-settings.png` (the title's settings screen keeps its layout).
- `just replay` unchanged. `just shot` md5 is now `e170997305a806320c6f9f967cf3cf5b` (the
  HUD has the button). `just shot-flow`: 3 more entities and 1 more text while playing
  (the button), the same counts in the level transition and on the game over.
