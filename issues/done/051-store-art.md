# [051] Store art - screenshots, feature graphic, preview video

**Status:** done
**Milestone:** M5
**Depends on:** [036], [037], [040], [050]
**Created:** 2026-11-02
**Closed:** 2026-11-03

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
- Done (2026-11-03): `just store-art` builds the game with `dev_tools` and runs
  `tools/store-art.py`, which writes into `target/store-art/` (not tracked: one run makes
  it again from the code):
  - `app-store/iphone/` 1206 x 2622 and `app-store/ipad/` 2064 x 2752, `play/phone/`
    1080 x 2160: three game shots and the wiki each, RGB PNG (no alpha).
  - `play/feature-graphic.png` 1024 x 500 RGB: the logo box between two dimmed Play
    shots; no text near an edge.
  - `play/icon-512.png` 512 x 512 RGBA, 49 KB (`tools/draw-icon.py`'s `draw`).
  The script checks each size and the icon's 1 MB limit.
- New dev tools for it: `RIX_LEVEL=<n>` (with `RIX_SEED`) starts the run at level n with
  that level's seed, as after a clear; `RIX_SHOT_SCENE=store` draws three lines from the
  field's fractions and takes the shot while the third is half drawn. `just shot` (md5
  e170997305a806320c6f9f967cf3cf5b) and `just replay` did not move.
- Seeds: the field takes the image shape, so a seed can play out differently per size.
  A sweep of seeds 1 to 30 at levels 6, 9 and 12 per size (RIX_STATE at tick 160: enemies
  and claimed fraction) found the picks; most seeds die to the arrow or claim the big part
  first. Skins do not move the game (same state hash with `RIX_SKINS`), so shots 2 and 3
  use laser + brick and chain + circuit for variety.
- Dead ends: a landscape game shot for the feature graphic (the field stays portrait, so
  it is a narrow strip); a title shot (the shot build is a desktop build and its title has
  QUIT, which phones do not have).
- Left out: Steam (D9: no Steam); the itch.io cover goes with [059]; the preview video is
  [100] (optional in both stores).
