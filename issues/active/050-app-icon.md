# [050] App icon set

**Status:** active
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
Android adaptive icon (foreground + background, monochrome for themed icons), and the
itch.io page and web favicon.

## Verification
The icon shows on the iOS simulator home screen, the Android emulator launcher (themed on
and off) and the itch.io page.

## Notes
- [011] Q6 (2026-11-02): a new design, generated with flux, PixelLab or similar; keep the
  prompt and seed tracked so it can be made again. The `feat/ios` icon is not used.
  Desktop icons only if [058] ships desktop downloads.
- D13 (2026-11-02): no desktop builds ship, so no `.icns` or `.ico`.
- 2026-11-03: four candidates on the picking page https://claude.ai/artifact/RLHHf3ivq5pLKKkmNoMn8t
  (store `picks/icon`): A "Logo box" and B "Cut, game colours" drawn by a PIL script
  (game palette, `assets/fonts/main.ttf`, glow, light scanlines); C "Cut, bright" and
  D "Shard" from EvoLink `doubao-seedream-5.0-pro` (`tools/image.py`, 2048 px PNG plus a
  sidecar JSON with the request and the task). Seedream takes no seed, so an AI pick ships
  its master PNG and sidecar as the tracked source ([011] Q6 asked for prompt and seed).
  No mobile project has an icon yet: Android uses the system default, iOS has no asset
  catalog.
