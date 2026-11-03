# [082] Juice follow-ups: level-clear sweep, line trail, CRT per device, shader warm-up

**Status:** done
**Milestone:** M4
**Depends on:** [040]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
[040] shipped the claim flash, the screen shake, the sparks and the item outline. Its
other ideas are still open: the level clear only fades the claimed shapes with a drift;
the drawn line has no trail; the CRT pass uses one set of parameters on every screen; the
custom pipelines (CRT, explosions, glow) compile on first use, which may hitch on a phone.

## Approach
- Level clear: a sweep of light across the claimed area or a burst of sparks along its
  edge (`fx::Burst`), before the "LEVEL N" banner.
- Line: a few sparks or a glow at the line's head while the finger draws.
- Per-kind death variants (a square shatters, an arrow spins away), in the enemy colour.
- CRT and shader warm-up wait on [025]'s device numbers: tune `PostProcessSettings` per
  device class if the scanlines or the grid look wrong on a phone; if the first kill or
  bomb hitches, spawn one hidden-but-drawn entity per material on `Screen::Loading`.

## Verification
`cargo test` and `just replay` pass; shots of a level clear and a line in progress in
Notes; frame time on the emulator stays within [024]'s budget.

## Notes
- Split out of [040] (2026-11-03).
- Scope (2026-11-03): the CRT per device and the shader warm-up wait on [025]'s device
  numbers; they moved to [086]. This issue did the juice that needs no device.
- Done 2026-11-03:
  - Line: the head of a line being drawn throws 3 cyan sparks back along the line, at most
    every 30 ms (`fx::spark_line_head`, from `DrawLine`). Sparks of 7 x 2 units, so a thin
    trail, not a flare.
  - Level clear: 3 slow sparks every 40 units along the `FreeRegion` edge on
    `OnEnter(Phase::LevelTransition)` (`fx::spark_level_clear`, `along_edge` tested), before
    the "LEVEL N" banner.
  - Deaths per kind: a square shatters into four quarter pieces that fly apart (160 u/s)
    and spin; an arrow's corpse spins away along its heading (140 u/s, 14 rad/s); the
    others keep the [039] pop (`Drift` in `enemies`). Found on the way: a kill took the
    enemy's colour with its alpha, so a square killed while faded out left an invisible
    corpse and invisible sparks; the kill colour is now opaque.
  - Not done: a sweep of light (the edge sparks do that job) and a glow at the line head
    (the sparks do it).
- Evidence: `cargo test` 100 passed (`edge_points_are_at_most_the_gap_apart_from_each_corner`,
  `a_square_shatters_into_four_pieces_and_an_arrow_spins_away`); clippy clean with and
  without `dev_tools`; `just replay` unchanged; `just shot` md5 unchanged
  (e170997305a806320c6f9f967cf3cf5b: the sparks are gone at shot time). Patched shots
  (reverted): mid-line (sparks trail the head), the flow's level clear (sparks along the
  edge under "LEVEL 2"), four squares and four arrows killed 0.15 s before the picture
  (pieces and spinning arrows).
- Frame time on the Android emulator (`dev_tools` APK, [024] log, update time): title
  median 18 ms; claims and the level clear median 20 to 22 ms, max 31 ms (07:12:51, the
  claim that cleared level 1, with the edge sparks); [024] recorded 25 to 45 ms at a claim
  and a 26 to 28 ms median. The updates over 33 ms come at the `adb screencap` calls. The
  same run showed [049]'s tutorial on Android: the first rule with the ghost finger, then
  the claim rule after the first claim (a run that died before a claim showed it again).
