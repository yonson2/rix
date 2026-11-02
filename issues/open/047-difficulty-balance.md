# [047] Difficulty curve and balance pass

**Status:** open
**Milestone:** M4
**Depends on:** [032], [044], [045], [046]
**Created:** 2026-11-02
**Closed:** -

## Problem
Difficulty is the enemy count curve in `enemies/rng.rs` (1 + level/3 up to level 10, …,
capped at 12) and a fixed 75% target; speeds are constants per kind. The last commit
("re-balance enemies, increase difficulty a bit") was tuned on desktop.

## Approach
One table per level band: enemy mix ([032]), speeds, target %, item rate. Tune on a phone
([025]). Record target times per level in Notes.

## Verification
Pedro plays levels 1-10 on a phone and signs off the curve in Notes.

## Notes
- **owner-run** sign-off.
- [012] Q3 (2026-11-02): endless levels, one death ends the run. The curve must keep
  growing past any level a player can reach.
