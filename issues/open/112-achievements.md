# [112] Medals as Game Center and Play Games achievements

**Status:** open
**Milestone:** M7
**Depends on:** [068], [111]
**Created:** 2026-11-05
**Closed:** -

## Problem
D39: the medals come with the leaderboards. Players see them in the platform's
achievements too.

## Approach
- Each medal ([111]) is an achievement on both platforms, through the [068] layer (an
  unlock call beside `submit_score`). A medal earned before sign-in or offline unlocks at
  the next sign-in.
- Owner-run: make the achievements (ID, name, text, a 1024 x 1024 icon, points) in App
  Store Connect and in the Play Console. A script draws the icons, as
  `tools/draw-icon.py` does.
- Check the current GameKit and Play Games Services docs when the work starts.

## Verification
On a TestFlight build and a Play test build, a medal earned in play shows as the
platform achievement (owner-run check). Desktop and web builds run with the no-op backend.

## Notes
- From the 2026-11-05 brainstorm (D39).
