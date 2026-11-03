# [036] Menu UI redesign

**Status:** done
**Milestone:** M4
**Depends on:** [026], [027], [011]
**Created:** 2026-11-02
**Closed:** 2026-11-03

## Problem
The menu is a "RIX" box that flickers in and "Tap to start playing." (`menu/mod.rs`). No
settings, no credits, no high score, no way to quit on desktop.

## Approach
A title screen in the game's look ([011]): animated logo, Play, Settings ([048]),
Credits ([052]), best score, Quit on desktop only. Touch targets of at least 44 pt. Built
from shared widgets in `theme/` ([026]).

## Verification
`cargo test` (the press and release rule of the buttons) passes; `just shot-menu` shows
the logo, PLAY and QUIT; on the Android emulator a tap on PLAY starts a run. The
phone taps are in [025].

## Notes
- todo.md: "menu ui".
- [011] Q1 (2026-11-02): keep the neon-vector look; references Geometry Wars and Hyper Light
  Drifter.
- From [026] (2026-11-02): `src/theme/` exists (only `palette` so far); the widgets go
  there. Check first whether Bevy UI on 0.20 draws after the CRT node (then UI nodes get no
  bloom and no CRT); that may be why the menu and HUD use world-space `Text2d`.
- Done 2026-11-03: `src/theme/widget.rs` holds the shared button (`button(label, font,
  y)`, a world-space neon box with a `Text2d` label, 260 x 76 world units, about 230 x
  67 pt on a 390 pt wide phone). A press and a release on the same button trigger the
  entity event `Activate`; the screen observes it. The title (`src/screens/title.rs`) has
  the RIX logo (flicker, then a slow breathe), PLAY and, on desktop only, QUIT. The tap
  anywhere is gone.
- Moved out: SETTINGS and the best score to [048], CREDITS to [052] (both need their
  screens and data first). The Verification changed with that: the desktop and iOS
  simulator manual runs need a window or a headless tap that simctl lacks, so the phone
  check is in [025].
- Answer to the Notes question: Bevy UI still draws after the CRT node on 0.20, so the
  widgets are world-space shapes and `Text2d` and get bloom and CRT.
