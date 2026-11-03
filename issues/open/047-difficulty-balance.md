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
- [019] (2026-11-02): the field is now 450 x 800 (D17; it was the 625 x 625 window). Speeds, sizes and spawn counts are unchanged and need the retune D17 asks for. Playable area 380 x 700 (was 555 x 525, 91 % of it).
- [074] (2026-11-02): progress is now the exact claimed fraction (1 - free area / box
  area). The old measure used a 380 x 730 box for a 380 x 700 field, so it read about 4 %
  low; the 75 % target now comes a little sooner. Lines also end at the boundary, not
  17.5 units before a wall.
- From [044] (2026-11-03): the Splitter spawns from level 2 with weight 1 (as each other
  kind); Queen 120 units/s, Soldiers 220, two Soldiers per Queen, a turn of up to 90
  degrees every 1 to 2.5 s. All are constants at the top of `splitter.rs`.
- From [045] (2026-11-03): the Shooter spawns from level 3, weight 1; first shot 0.5 s into
  a line, then one every 1 s; bullets 140 units/s. A quick swipe (under 0.5 s) draws no
  fire, so the Shooter punishes slow lines. Constants at the top of `shooter.rs`.
- [046] items to tune (2026-11-03): first levels in `ITEM_LEVELS` (bomb 1, multiplier 2,
  time stop 3, auto-line 4) and the even pick; `BOOST_SECS` 10 and `BOOST_FACTOR` 2;
  `STOP_SECS` 4; token size 40 (the bomb's collider) and 7 s life. Check whether frozen
  enemies need a tint.
- 2026-11-03 ([089], D26): confirm the level that unlocks the soundtrack, now 3
  (`save::SOUNDTRACK_LEVEL`), before the first skin at 5.
