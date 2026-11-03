# [105] Native debug symbols for store crash reports (iOS dSYM, Play)

**Status:** open
**Milestone:** M5
**Depends on:** [053], [054]
**Created:** 2026-11-03
**Closed:**

## Problem
The first App Store upload (0.4.28, 428) warned: "Upload Symbols Failed. The archive did
not include a dSYM for the Rix.app". `mobile/ios/build_rust_deps.sh` builds the `rix`
binary with cargo and `lipo`s it into the app, so Xcode links nothing and makes no dSYM
(`target/ios/Rix.xcarchive/dSYMs` is empty). `[profile.release]` also keeps no debug
info. Crash reports from TestFlight and the App Store then show addresses, not function
names. Play Console gives the same warning for native code without symbols ([054]).

## Approach
- iOS: give the release build line tables (`debug = "line-tables-only"` in
  `[profile.release]`, or only for the iOS cargo line), run `dsymutil` on the lipo'd
  binary into `$DWARF_DSYM_FOLDER_PATH/$DWARF_DSYM_FILE_NAME`, then strip the binary.
- Android: upload the symbols with the AAB (`ndk.debugSymbolLevel` in Gradle) when [054]
  builds it.
- Check the app size before and after (the wasm profile strips debug info, keep that).

## Verification
`just ios-archive`, then `dwarfdump --uuid` on `target/ios/Rix.xcarchive/dSYMs/Rix.app.dSYM`
and on the app binary print the same UUIDs; the next `just ios-upload` has no "Upload
Symbols Failed" warning.

## Notes
