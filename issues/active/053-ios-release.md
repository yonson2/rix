# [053] iOS release - signing, App Store Connect, TestFlight

**Status:** active
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
- 2026-11-03: the listing text (short and full description) is `docs/store/listing.md`;
  the art comes from `just store-art` (`target/store-art/`).
- 2026-11-03, build ready for upload (agent):
  - `Info.plist`: `ITSAppUsesNonExemptEncryption` false (no networking, no own crypto),
    and `NSPhotoLibraryAddUsageDescription`: the share sheet gets the death picture as a
    PNG file and offers "Save Image", and without that key iOS stops the app on it.
  - `ios-src/PrivacyInfo.xcprivacy` (in the Resources phase): no tracking, no collected
    data; the release binary (`nm -u`) uses `stat`, `fstat`, `lstat` (file timestamps,
    reason C617.1) and `mach_absolute_time` (system boot time, 35F9.1); no disk space,
    user defaults or keyboard APIs. Scan again after a dependency change.
  - `just ios-archive` (Release, `target/ios/Rix.xcarchive`, signed by team
    `B8Y7SGQP5V` with automatic signing) and `just ios-upload` (export with
    `mobile/ios/ExportOptions.plist`: app-store-connect, destination upload, the Apple
    account in Xcode). The archive: 0.4.27 (427), UIDeviceFamily [1], iOS 15.0, icon
    without alpha, 118 MB.
  - The upload is owner-run: the agent's permission rules stop a send to App Store
    Connect.
  - First upload (2026-11-03, 0.4.28 (428)): the export failed twice. (1) "Your session
    has expired": sign in again in Xcode > Settings > Accounts. (2) "exportArchive Copy
    failed": Xcode runs `/usr/bin/rsync -E` (openrsync), which starts its other end as
    `rsync` from PATH, Homebrew's rsync 3.5.1, and that one does not know
    `--extended-attributes` (IDEDistributionPipeline.log). `just ios-upload` now puts
    `/usr/bin` first in PATH.
  - With `/usr/bin` first, the upload succeeded (2026-11-03 19:15): 0.4.28 (428) is in
    App Store Connect. One warning: no dSYM for the Rust binary, filed as [105].
  - 2026-11-03 23:40: 1.0.0 (10000) uploaded, with its dSYM ([105]). Tested on TestFlight.
  - 2026-11-04 10:00: 1.0.1 (10001) uploaded, with the fixes [106] and [107]. It replaces
    1.0.0 as the build for the review; the version page must say 1.0.1.
- 2026-11-05: App Review answered 1.0.1 with "Guideline 2.1 - Information Needed": a screen
  recording on a physical device from launch, and answers on purpose, setup, external
  services, regions and regulated material in the App Review Notes. The text is in
  `docs/store/listing.md` ("App Review notes"). Owner-run: record the video, paste the
  notes, attach the video, resubmit.
