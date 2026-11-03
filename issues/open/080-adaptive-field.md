# [080] The field takes the screen shape

**Status:** open
**Milestone:** M2
**Depends on:** [019]
**Created:** 2026-11-03
**Closed:** -

## Problem
Pedro, 2026-11-03: "I don't like how now you can have wasted screen real-state since the
change to a fixed size to guarantee reproducibility of a seed. I don't mind that a seed
will be slightly different on a different screen, I just care that a seed will influence
the rng ticks the same way [...] adapting to use all the screen on mobile seems like a
much bigger win for playability than what we have now".

Today `wall::FIELD` is a fixed 450 x 800 (D17, [019]) and the camera letterboxes it. A
19.5:9 phone shows empty bands above and below the field.

## Approach
D21 (2026-11-03, Pedro's answers):
- The field is 450 world units wide. Its height follows the window shape at the start of
  each run: `450 * height / width`, clamped from 600 (3:4, a portrait tablet) to 1050
  (9:21). Outside the clamp, the camera letterboxes as now.
- Phones are portrait only: remove the landscape entries from the iOS `Info.plist`; set
  `android:screenOrientation="portrait"` in the manifest. Android 16 ignores the
  orientation on large screens (600 dp and wider) for apps that target API 36, so a
  tablet can still be landscape: the clamp letterboxes it.
- A `Field` resource replaces the `FIELD` constant (about 20 uses in 10 files: walls,
  spawn bounds, the item despawn line, input clamp, pause overlay, HUD, camera, dev
  tools). Pure functions take the size as an argument. The walls and the free region
  are built from it per run, as now.
- The camera `ScalingMode::AutoMin` takes the run's field size. A resize or rotation
  mid-run keeps the field until the next run (letterbox meanwhile).
- The RNG draws stay the same for a seed; positions follow the field size.
- Safe areas: the top wall (65 units) holds the HUD, and on a phone with a notch or a
  Dynamic Island it sits under the cutout. Find the safe-area insets (winit or Bevy API;
  else the platform values) and keep the HUD inside them; the walls may run under the
  cutout.
- Desktop and web: the field follows the window inside the clamp (the desktop window
  stays 450 x 800 by default, so the default look does not change).
- Docs: D17 text in `ARCHITECTURE.md` and [019] facts, the M2 goal ("play the same on
  every screen size").

## Verification
A unit test of the size rule (aspect to height, both clamps); `just shot` at the default
450 x 800 gives the same md5 as before and `just replay` passes;
`RIX_SHOT_SIZE=390x844 just shot` shows the field over the whole image; Android emulator
and iOS simulator screenshots show the field over the whole screen with the HUD clear of
the cutout, and the app stays portrait after `adb shell settings put system
user_rotation 1` with auto-rotate off.

## Notes
- D21 replaces D17 (fixed 9:16) and the [010] Q4 answer (both orientations).
- Pedro picked the fixed width over "1 unit = 1 point" (an iPad would get about 3x the
  field with same-size enemies) and over a fixed area (objects bigger on tall phones).
- Spawn counts stay per level, not per area: a taller field is a little sparser. Retune
  in [047] if needed.
