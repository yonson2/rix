# [103] Store shots: wobbly finger lines, a line set per shot

**Status:** done
**Milestone:** M5
**Depends on:** [051], [101]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Pedro (2026-11-03): "the lines that you draw between the walls need to be more curvy and
wobbly, emulating ... the movement of a finger. So like sine waves or something like
that. And also, the screenshots shouldn't share the same pattern." The store scene
([051]) draws the same three straight lines in every game shot.

## Approach
- `src/dev_tools.rs`: the store scene has three line sets of three lines.
  `RIX_SHOT_LINES=<n>` picks the set (default 0). Each line sways off the straight path
  as a sum of sines that are zero at both ends, so the line still starts and ends where
  the straight one did.
- `tools/store-art.py`: one set per game shot; new seeds per set and size (the lines
  move the enemies, so the old seeds can die).

## Verification
`just store-art` makes all shots; the three game shots show curved lines in three
different patterns; `just shot` md5 and `just replay` do not change (the store scene only).

## Notes
- Line sets (fractions of the half field): 0 = across the bottom, down from the top
  right, across from the left (the old lines); 1 = a slant from the left wall, up from the
  bottom, in from the right; 2 = up from the bottom left, in from the right, up from the
  bottom right. Lines take 32 ticks (was 24); the shot is at step 20.
- Dead end: a line that starts at the top left (set 2's first line went down from
  x = -0.25) starts on the HUD pause button, so the press starts no line ([073]). That
  line goes up now.
- Sweep (seeds 1 to 20, levels 6 and 9, both sizes, per set): kept the runs that claim
  25 to 65 % with three or more enemies at both sizes. Picks: shot 1 set 2 seed 14
  level 6; shot 2 set 0 seed 13 level 9 (STAR on, `laser,brick`); shot 3 set 1 seed 4
  level 6 (`chain,circuit`).
- Seed 1 level 9 set 1 claimed without skins but died with `chain,circuit` at
  1080x2160: a wall skin changed the run. Filed [104].
- Checks: `just store-art` makes all shots at the right sizes (RGB); fmt, clippy (with
  and without `dev_tools`), 131 tests; `just shot` md5 `e170997305a806320c6f9f967cf3cf5b`
  (unchanged); `just replay` passes with the same hashes.
