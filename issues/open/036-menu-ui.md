# [036] Menu UI redesign

**Status:** open
**Milestone:** M4
**Depends on:** [026], [027], [011]
**Created:** 2026-11-02
**Closed:** -

## Problem
The menu is a "RIX" box that flickers in and "Tap to start playing." (`menu/mod.rs`). No
settings, no credits, no high score, no way to quit on desktop.

## Approach
A title screen in the game's look ([011]): animated logo, Play, Settings ([048]),
Credits ([052]), best score, Quit on desktop only. Touch targets of at least 44 pt. Built
from shared widgets in `theme/` ([026]).

## Verification
`just shot-menu` shows the new menu; each button opens its screen in a manual run on
desktop and on the iOS simulator.

## Notes
- todo.md: "menu ui".
- [011] Q1 (2026-11-02): keep the neon-vector look; references Geometry Wars and Hyper Light
  Drifter.
