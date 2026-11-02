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
