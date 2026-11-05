# [125] Challenge and daily leaderboards

**Status:** open
**Milestone:** M9
**Depends on:** [068], [109], [123], [124]
**Created:** 2026-11-05
**Closed:** -

## Problem
Challenge scores do not go to the main board ([109] Q3). Players may want to compare them.

## Approach
If [109] Q5 asks for boards: one board per curated challenge, and a daily board if there
is a daily seed (Game Center has recurring leaderboards; Play Games boards have a daily
view), through the [068] layer. Owner-run: the board IDs in both stores.

## Verification
On a TestFlight build and a Play test build, a challenge score shows on its board
(owner-run check).

## Notes
- From the 2026-11-05 brainstorm (D39); waits for [109] Q5.
