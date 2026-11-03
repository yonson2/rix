# [070] Progressive unlocks with a tip on first sight

**Status:** open
**Milestone:** M4
**Depends on:** [032], [044], [045], [046], [069], [048], [049]
**Created:** 2026-11-02
**Closed:** -

## Problem
Pedro's idea (2026-11-02, D18): once all enemies and items exist and the game is about
ready, a new player should not meet everything at once. Not every enemy and item is
unlocked at the start, and the first time an element of a kind spawns, a short tip
explains it.

## Approach
An unlock list in the saved data ([048]): which enemy and item kinds the player has met.
The spawn table ([032]) only draws unlocked kinds, plus a rule that unlocks the next
kind (by level reached, or by run count; decide with Pedro). On the first spawn of a kind,
pause briefly and show one tip line with the element highlighted (reuse the tutorial
overlay of [049]). Seeds ([067]): decide whether unlocks change what a seed spawns (a
shared seed must give the same game, so a seed run may need everything unlocked).

## Verification
A fresh save: level 1 spawns only the starter kinds; the first spawn of a new kind shows
its tip once; a second run does not show it again (test on the unlock list and a scripted
run).

## Notes
- Overlaps [049] (the rules tutorial): [049] teaches the line and the claim; this issue
  teaches each enemy and item.
- From [049] (2026-11-03): the tutorial is `src/game/tutorial.rs` (overlay with one rule
  text, a ghost finger and SKIP; a flag in `Save`). A tip here can reuse its overlay
  pattern; [049] does not pause the game, a tip that does needs its own rule.
