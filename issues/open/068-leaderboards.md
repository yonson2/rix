# [068] Leaderboards on Game Center and Play Games

**Status:** open
**Milestone:** M4
**Depends on:** [020], [021], [048]
**Created:** 2026-11-02
**Closed:** -

## Problem
[012] Q5: v1 has a leaderboard. It must be platform-agnostic in the game code and submit to
Game Center on iOS and to Play Games on Android.

## Approach
One small plugin with a `submit_score` call and a "show leaderboard" action. One backend
per platform behind `cfg(target_os)`: iOS calls GameKit (sign-in, submit, show), Android
calls Play Games Services v2 through the Android project (JNI). Web and desktop get a
no-op backend. One leaderboard: best score of a run. The store side (a Game Center
leaderboard in App Store Connect, a Play Games Services project in the Play Console) is
owner-run and belongs to [053] and [054]. Check current crates for GameKit and Play Games
bindings when the work starts.

## Verification
On a TestFlight build and a Play internal-test build, a finished run's score shows in the
platform leaderboard (owner-run check). Desktop and web builds run with the no-op backend.

## Notes
- From [012] Q5 (Pedro, 2026-11-02).
