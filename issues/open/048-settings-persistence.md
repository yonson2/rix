# [048] Settings, high score and stats persistence

**Status:** open
**Milestone:** M4
**Depends on:** [028], [041]
**Created:** 2026-11-02
**Closed:** -

## Problem
Nothing persists: no best score, no settings. todo.md: "STATS resource (keep track of
enemies killed, and think more...)".

## Approach
A small save file in the platform's app data directory (iOS Application Support, Android
internal storage, desktop config dir): best score, lifetime stats (claims, kills, levels,
time), settings (music and SFX volume, CRT intensity, reduced flashing, haptics). A
settings screen reachable from the menu ([036]) and the pause menu ([034]).

## Verification
Set a best score and change a setting, restart the app: both are kept (desktop and iOS
simulator).

## Notes
- todo.md: "STATS resource (keep track of enemies killed, and think more...)".
- From [036] (2026-11-03): the title has PLAY (and QUIT on desktop) as
  `theme::widget::button`s. This issue adds a SETTINGS button and the best score to the
  title (`src/screens/title.rs`; buttons at y -60 and -170, the logo at 170).
- From [038] (2026-11-03): the game over screen (`src/screens/game_over.rs`, world
  `Text2d`) gets the best score and a "NEW BEST" mark here.
