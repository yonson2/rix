# [093] Laser line rework: a hot beam

**Status:** done
**Milestone:** M4
**Depends on:** [078]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Pedro (D27): "The laser line needs a rework, its not cool enough." Today it is a red line
with static bright dashes.

## Approach
Pedro picked "Hot beam": a white-hot thin core, a wide red glow, light pulses that run
along the line to the finger, a slight flicker, and a bright dot at the tip. Look only
(draw systems in `Update`; no collider or gameplay change), so replays do not move. The
Star's rainbow still wins while it lasts.

## Verification
Shots of the laser line mid-line (a few frames apart, to see the pulses move) look like a
hot beam; `just replay` unchanged with `RIX_SKINS=laser,plain`; `cargo test`, clippy.

## Notes
- Done 2026-11-03. In `skins::add_marks`, a laser segment gets `Beam { along }` and a soft
  red strip in place of its sprite (`glow_strip`, 14 wide, a Gaussian across, HDR
  `LASER_RED`), a white-hot core child (1.5 wide) and, on the newest segment only, a
  glowing dot (`LaserTip`; the cursor despawns the old one). `pulse_laser` (`Update`, after
  `update_sprite_color`; not in a Star or a death, so the rainbow and the death flash win)
  lights each segment by `pulse(along, secs)` (crests 90 apart, 420 units/s to the
  finger) and a flicker, and the tip breathes. The old dashes are gone.
- The skins screen draws its line in 34 pieces, so the preview pulses too.
- Evidence: test `the_laser_pulses_run_to_the_finger`; two mid-line shots 0.05 s apart (the
  crests move about 21 units to the finger); the skins screen shot; `just replay`
  unchanged, also with `RIX_SKINS=laser,plain`; clippy clean.
