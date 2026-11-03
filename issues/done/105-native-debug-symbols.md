# [105] Native debug symbols for store crash reports (iOS dSYM, Play)

**Status:** done
**Milestone:** M5
**Depends on:** [053], [054]
**Created:** 2026-11-03
**Closed:** 2026-11-03

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
- 2026-11-03: `[profile.release] debug = "line-tables-only"`. Android: cargo-ndk strips
  the library when it copies it to `jniLibs` (665 MB to 86 MB), so Gradle saw no symbols;
  `just android-release` now passes `--no-strip`. Gradle could not strip either ("Unable to
  strip the following libraries"): AGP 9.1's default NDK is not installed, so
  `app/build.gradle` sets `ndkVersion '26.1.10909125'` (the one cargo-ndk uses). FULL would
  add some 525 MB of DWARF (`.debug_str` 214, `.debug_info` 167, `.debug_ranges` 88,
  `.debug_line` 54 MB), so the level is `SYMBOL_TABLE` (function names): the AAB has
  `BUNDLE-METADATA/com.android.tools.build.debugsymbols/arm64-v8a/librix_mobile.so.sym`
  (136 MB), and the shipped library keeps its size (85.77 MB).
- 2026-11-03, iOS: `build_rust_deps.sh` runs `dsymutil` on the app binary in Release (Cargo
  keeps the object files on macOS, `split-debuginfo = "unpacked"`), then `strip`s it: Xcode
  strips nothing it did not link. The 1.0.0 (10000) archive: `dwarfdump --uuid` prints
  `468363E9-0C14-3938-92AE-5F37C4C199EE` for the dSYM (637 MB) and the binary; the binary
  is 77.6 MB (0.4.28: 118.7 MB unstripped; with the debug map, before the strip, 133.1 MB);
  `codesign --verify --deep --strict` passes. Still to check: the 1.0.0 upload has no
  "Upload Symbols Failed" warning, and the stripped build runs from TestFlight.
- 2026-11-03 23:40: the 1.0.0 (10000) upload succeeded with no "Upload Symbols Failed"
  warning. Verification passes; closed. The stripped build's first run on a phone is part
  of [053] (TestFlight) and [055].
