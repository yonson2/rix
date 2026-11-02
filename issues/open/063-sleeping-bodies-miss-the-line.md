# [063] Resting enemies and items may sleep and miss the drawn line

**Status:** open
**Milestone:** M3
**Depends on:** [018]
**Created:** 2026-11-02
**Closed:** -

## Problem
Read from the avian source during [015], not yet reproduced. A dynamic body that rests
for `TimeToSleep` (0.5 s by default) below `SleepThreshold` (0.15) falls asleep. The broad
phase skips pairs where both sides are inactive (static or sleeping), and the drawn line
(`TempLineSegment`) is a collider without a rigid body, so it counts as static. A sleeping
body does not wake when a collider appears next to it. So:
- a square that rests on a wall or a claimed area may not kill the player when the line
  touches it;
- a bomb (`GravityScale(0.)`, no start velocity) may never activate when the line crosses
  it.
Same rule in avian 0.3.1 and 0.4.1 (`collision/broad_phase.rs`, `IS_INACTIVE`), so this is
older than the upgrade.

## Approach
Reproduce first: a scripted run (dev harness, [006]) that waits until a square or bomb
rests, then draws the line through it. If it reproduces, add `SleepingDisabled` to enemies
and items (few bodies, small cost), or wake them when a line spawns.

## Verification
A scripted run draws the line through a resting square (expects game over) and through a
resting bomb (expects the bomb to activate).

## Notes
