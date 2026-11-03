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
- 2026-11-03, rules read (Pedro asked for the steps): a personal account needs a closed
  test with at least 12 testers opted in for the last 14 days continuously before it can
  apply for production
  (https://support.google.com/googleplay/android-developer/answer/14151465). New apps
  must target API 36 since 2026-08-31
  (https://developer.android.com/google/play/requirements/target-sdk): the project has
  `targetSdk 36`. This test is the longest step of both stores: start it first.
- Owner steps, in order: (1) Play Console personal account (25 USD once, identity check).
  (2) The app: name, English, game, free. (3) App content: privacy URL, no ads, app access
  (all open), content rating questionnaire (IARC), target audience, Data safety (no data
  collected). (4) Store listing: short and full text, icon 512, feature graphic, phone
  shots ([051]). (5) Play App Signing (Google keeps the app key) and an upload key: a
  keystore outside git, with a backup; the agent can make it and the signed AAB recipe.
  (6) Closed test: upload the AAB, add 12 or more testers (a list or a Google group),
  each opts in and installs; keep the test running 14 days. (7) Apply for production,
  then the staged release ([056]). (8) Play Games Services: one leaderboard, its IDs to
  [068].
- D36: when the Play page exists, add its link to `site/index.html` and run
  `just site-deploy`.
