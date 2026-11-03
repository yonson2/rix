# [050] App icon set

**Status:** done
**Milestone:** M5
**Depends on:** [011]
**Created:** 2026-11-02
**Closed:** 2026-11-03

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
- 2026-11-03 (D31): Pedro picked A, the logo box, with no note (store `picks/icon`).
  `tools/draw-icon.py` draws it (no randomness, so it is its own source) and writes:
  - iOS: `mobile/ios/ios-src/Assets.xcassets/AppIcon.appiconset/icon-1024.png` (RGB, no
    alpha; the single-size catalog) with its `Contents.json`; the Xcode project gets the
    catalog in the Resources phase and `ASSETCATALOG_COMPILER_APPICON_NAME = AppIcon`.
  - Android (minSdk 31, so only the adaptive icon): `mipmap-<density>/
    ic_launcher_foreground.png` and `ic_launcher_monochrome.png` (108 dp layers, the box
    diagonal at 97 % of the 66 dp safe circle), `mipmap-anydpi-v26/ic_launcher.xml`,
    `values/ic_launcher_background.xml` (#15181A); the manifest names `@mipmap/ic_launcher`
    as icon and round icon. The foreground's colours are solved so that over the
    background it matches the iOS icon (its glow keeps working on any launcher).
  - Web favicon and itch.io art: no web build yet; they move to [058] and [059], which
    can call the same drawing at their sizes.
  - The B, C and D designs are not kept: C and D (AI) lived only in the picking page and
    the scratchpad; `tools/image.py` stays for later art.
- Verification 2026-11-03: Android emulator (Pixel launcher, API 36): the app list shows
  the box in the round mask; with themed icons on (Wallpaper & style, turned on with
  `uiautomator`, then off again), the home screen shows the monochrome layer tinted like
  the system icons. iOS simulator: the built `Rix.app` has `CFBundleIconName = AppIcon`,
  `Assets.car` and `AppIcon60x60@2x.png`, which is the box. The simulator home screen
  puts Rix on page 2 and takes no headless swipe, so the home screen itself was not
  seen; [055] covers it on a phone. The itch.io page moves to [059].
