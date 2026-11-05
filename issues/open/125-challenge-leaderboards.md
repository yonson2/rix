# [125] Challenge and daily leaderboards

**Status:** open
**Milestone:** M10
**Depends on:** [068], [109], [123], [124]
**Created:** 2026-11-05
**Closed:** -

## Problem
Challenge scores do not go to the main board ([109] Q3). Players may want to compare them.

## Approach
[109] Q5: a board per challenge. One board per curated challenge (ten), and a daily board
(Game Center has recurring leaderboards; Play Games boards have a daily view), through the
[068] layer. Owner-run: the board IDs in both stores. Check each store's leaderboard limit
when the work starts.

## Verification
On a TestFlight build and a Play test build, a challenge score shows on its board
(owner-run check).

## Notes
- From the 2026-11-05 brainstorm (D39); [109] Q5: per challenge.
- 2026-11-05 (D40): moves to M10, after M7 to M9, so every scoring change comes
  before the boards go live and the store setup is one session.
