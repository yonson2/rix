# [050] App icon set

**Status:** open
**Milestone:** M5
**Depends on:** [011]
**Created:** 2026-11-02
**Closed:** -

## Problem
Main has no icon. `feat/ios` has a 1024 px `icon.png` (plus 120 and 152) and a progress
commit "progress towards app icon"; `feat/android` has Bevy's default launcher icon.

## Approach
One master design ([011] Q6), drawn reproducibly (a script like ketania's
`tools/draw-icon.py`, or a tracked source file) into: iOS asset catalog (1024, no alpha),
Android adaptive icon (foreground + background, monochrome for themed icons), desktop
(`.icns`, `.ico`, window icon).

## Verification
The icon shows on the iOS simulator home screen, the Android emulator launcher (themed on
and off) and the desktop window/dock.

## Notes
- [011] Q6 (2026-11-02): a new design, generated with flux, PixelLab or similar; keep the
  prompt and seed tracked so it can be made again. The `feat/ios` icon is not used.
  Desktop icons only if [058] ships desktop downloads.
