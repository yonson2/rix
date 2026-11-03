# [101] Store captions and iPhone only

**Status:** done
**Milestone:** M5
**Depends on:** [051]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Pedro on [051]'s shots (D32): "Add captions" and "let's go iphone only". Raw game shots do
not say what the game is; the iPad set is not needed once the app does not run as an iPad
app.

## Approach
- Xcode: `TARGETED_DEVICE_FAMILY = "1"` (iPhone) in both build configurations.
- `tools/store-art.py`: no iPad size; each screenshot becomes a caption in the game font
  over the dark ground, with the game shot below it in a glowing frame (the logo box's
  edge). The raw shots stay in `target/store-art/raw/` for the feature graphic.

## Verification
`just store-art` writes captioned iPhone (1206 x 2622) and Play (1080 x 2160) sets and no
iPad set; the iOS build for the simulator passes with the iPhone-only target.

## Notes
- Done (2026-11-03): `TARGETED_DEVICE_FAMILY = 1` in both configurations; the simulator
  build passes and the app's Info.plist has `UIDeviceFamily` [1] (it had 1 and 2).
  `just store-art` writes the iPhone and Play sets, each shot under a two-line caption in
  the game font (one letter size for all, fitted to the longest line), framed with the
  logo box's glowing edge; no iPad set; raw shots in `target/store-art/raw/`.
- Captions (mine, Pedro may change them in `CAPTIONS`): "DRAW A LINE / CLAIM THE FIELD",
  "GRAB POWER-UPS / DODGE THE SWARM", "UNLOCK NEW / LINES AND WALLS", "KNOW YOUR /
  ENEMIES".
