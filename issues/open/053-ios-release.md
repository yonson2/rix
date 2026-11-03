# [053] iOS release - signing, App Store Connect, TestFlight

**Status:** open
**Milestone:** M5
**Depends on:** [020], [025], [050], [052], [010]
**Created:** 2026-11-02
**Closed:** -

## Problem
No release pipeline for iOS: no signing setup, no archive, no App Store Connect record.

## Approach
Release scheme and archive from `mobile/ios/` (`just ios-archive`), privacy manifest
(`PrivacyInfo.xcprivacy`), export compliance (no encryption beyond the OS), App Store
Connect record with the bundle id from [010], TestFlight build, listing text and art
([051]). Check the current minimum Xcode/SDK rule for uploads when the work starts.

## Verification
A build is live in TestFlight and installs on Pedro's iPhone.

## Notes
- Account parts are **owner-run**.
- [010] answers (2026-11-02): bundle id `top.cosas.rix`; free, no in-app purchases; English
  listing only. The Game Center leaderboard is set up here ([068]).
- From [052] (2026-11-03): fill the store's age-rating questionnaire (owner-run): no
  violence beyond abstract shapes, no ads, no purchases, no data collected; privacy URL
  https://rix.cosas.top/privacy ([102], D33).
- 2026-11-03, rules read (Pedro asked for the steps): since 2026-04-28 an upload must be
  built with Xcode 26 or later and the iOS 26 SDK
  (https://developer.apple.com/news/upcoming-requirements/); this Mac has Xcode 27.
  Not in the project yet: `PrivacyInfo.xcprivacy`, `ITSAppUsesNonExemptEncryption = NO`,
  a release archive recipe.
- Owner steps, in order: (1) Apple Developer Program, individual (99 USD a year, Apple ID
  with two-factor sign-in). (2) App Store Connect: the EU trader status (DSA; a trader's
  address and phone show in the EU store), the app record (bundle id `top.cosas.rix`,
  the name: "Rix" may be taken, SKU, English). (3) App Privacy "Data Not Collected", age
  rating questionnaire, category Games > Arcade, free, privacy URL, support URL. (4) Game
  Center: one leaderboard, its ID to [068]. (5) Upload: Xcode Organizer, or an App Store
  Connect API key (outside git) so the agent uploads. (6) TestFlight on Pedro's iPhone,
  then the review ([056]).
- D36: when the App Store page exists, add its link to `site/index.html` and run
  `just site-deploy`.
