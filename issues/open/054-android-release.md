# [054] Android release - AAB, signing, Play Console, closed test

**Status:** open
**Milestone:** M5
**Depends on:** [021], [022], [025], [050], [052], [010]
**Created:** 2026-11-02
**Closed:** -

## Problem
No Android release build: no signing, no AAB, no Play listing. New personal Play
developer accounts must run a closed test with testers for 14 days before production
(check the current tester count).

## Approach
`just android-release` builds a signed AAB (arm64-v8a, plus armeabi-v7a if [010] Q6 wants
old phones), signing keys read from an ignored properties file, targetSdk at the current
Play requirement, Play App Signing, Data safety form (no data collected, if true), listing
and art ([051]), closed test track.

## Verification
The signed AAB is accepted by the Play Console and installs from the closed test track on
a phone.

## Notes
- Account parts are **owner-run**.
- [010] answers (2026-11-02): application id `top.cosas.rix`; free, no ads; English listing
  only; arm64-v8a only (a newer floor is fine). The Play Games leaderboard is set up here
  ([068]).
- From [052] (2026-11-03): fill the store's age-rating questionnaire (owner-run): no
  violence beyond abstract shapes, no ads, no purchases, no data collected; privacy URL
  https://rix.cosas.top/privacy ([102], D33).
