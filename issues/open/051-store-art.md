# [051] Store art - screenshots, feature graphic, preview video

**Status:** open
**Milestone:** M5
**Depends on:** [036], [037], [040], [050]
**Created:** 2026-11-02
**Closed:** -

## Problem
Each store needs its own set: App Store screenshots per device class, Play screenshots and
a 1024x500 feature graphic, Steam capsules and screenshots, itch.io cover.

## Approach
Generate the screenshots from the game itself with the harness ([006]) at each required
resolution and seed, so they can be redone after any visual change; a short preview video
from a scripted run. Check each store's current size list when the work starts.

## Verification
A folder with every required asset per store, sizes checked against each store's current
spec (links in Notes).

## Notes
- [011] Q1 (2026-11-02): the store art follows the neon look; references Geometry Wars and
  Hyper Light Drifter.
- Store specs read 2026-11-03 (check again when the work starts):
  - App Store: https://developer.apple.com/help/app-store-connect/reference/screenshot-specifications/
    The newer page lists one required iPhone class (1179 x 2556 or 1206 x 2622); an older
    page lists 6.9-inch (1260 x 2736, 1290 x 2796, 1320 x 2868). PNG or JPEG, up to 10.
    The live Media Manager says which it requires. iPad only if Rix supports iPad
    (`TARGETED_DEVICE_FAMILY = "1,2"` today).
  - Play: https://support.google.com/googleplay/android-developer/answer/9866151
    Feature graphic 1024 x 500 (JPEG or 24-bit PNG, no alpha, no text that a crop can
    cut); 2 to 8 phone screenshots, 24-bit PNG or JPEG, sides 320 to 3840 px, long side
    at most twice the short one (portrait 1080 x 1920 or more recommended, so a 9:19.5
    phone shot must be cropped or framed to 1:2); hi-res icon 512 x 512, 32-bit PNG,
    under 1 MB (`tools/draw-icon.py` draws it, [050]).
