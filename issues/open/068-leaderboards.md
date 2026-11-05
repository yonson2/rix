# [068] Leaderboards on Game Center and Play Games

**Status:** open
**Milestone:** M10
**Depends on:** [020], [021], [048], [053], [054]
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
- 2026-11-03 (D25): waits for Pedro's store accounts: the App Store Connect leaderboard ID
  ([053]) and the Play Games app ID ([054]). The code starts when they exist.
- 2026-11-03 (from [053]/[054] steps): scores then go to Game Center and Play Games. Before
  the store review, check the App Privacy and Data safety answers ("no data collected")
  and `site/privacy.html` again.
- 2026-11-03 (D37): after 1.0. 1.0 ships without leaderboards; this is the next work (1.1).
- 2026-11-05 (D39): moves to M7 (1.1), with the medals ([111], [112]) and the score
  bonuses ([110]). The bonuses go in before the boards go live.
- 2026-11-05 (D40): moves to M10, after M7 to M9, so every scoring change comes
  before the boards go live and the store setup is one session.
- At the first sign-in, consider sending the saved best score, so play before 1.4 counts.
