# [095] Share: the score and a picture of the death

**Status:** done
**Milestone:** M4
**Depends on:** [067], [038]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Pedro (D27): "when sharing a high score we should make it more fun to share. Include the
score and maybe, a screenshot of the state of the canvas when one died? (So you can see
the enemies and what killed you?" Today SHARE sends "Rix seed CODE".

## Approach
- The text: the score, the level and the seed code (for example "I scored 12,345 on level 7
  in Rix! Seed: CODE").
- At the death (`OnEnter(IsPaused::Dying)`), a `Screenshot` of the window keeps the frame
  (with the HUD and the CRT) in memory; no capture without a window (shot runs).
- SHARE on a phone writes it as a PNG to the app's cache and passes it to the share sheet
  with the text: Android through a `FileProvider` content URI (`ACTION_SEND`, `image/png`,
  `EXTRA_STREAM` and `EXTRA_TEXT`), iOS as a `UIImage` next to the text. Off phones, COPY
  copies the text only.

## Verification
A unit test of the share text; on the emulator, SHARE opens the sheet with the picture and
the text; `cargo test`, clippy, `just android`.

## Notes
- Done 2026-11-03. `death::capture` (phones only, `share::SHEET`) takes a `Screenshot` of
  the window on `OnEnter(IsPaused::Dying)` and keeps it as `DeathPicture` (an old one is
  dropped first, so a failed capture never shares an old death). `Share { text, picture }`;
  `game_over::share_text`: "I scored N on level L in Rix! Seed CODE". `share::write_png`
  writes the picture (RGB) on the SHARE tap: Android `files/share/rix.png`, served by a
  `FileProvider` (`${applicationId}.share`, `res/xml/share_paths.xml`), and
  `MainActivity.share(text, picturePath)` sends `image/png` with `EXTRA_STREAM`,
  `EXTRA_TEXT` and a `ClipData` for the preview; iOS writes to the temp directory and adds
  an `NSURL` file URL to the activity items (`objc2-foundation` feature `NSURL`).
- Evidence: test `a_share_tells_the_score_the_level_and_the_seed`; 127 tests; clippy clean
  with and without `dev_tools`; `cargo check --target aarch64-apple-ios-sim` clean; on the
  emulator a run to a death, then SHARE opens "Sharing image" with the picture's thumbnail
  and "I scored 70 on level 1 in Rix! Seed 0494-MSDD"; the PNG pulled from the app
  (`run-as top.cosas.rix cat files/share/rix.png`) is the death frame (the line, the
  square that hit it, the HUD).
- Not checked: the iOS sheet at run time (the simulator has no touch automation here); it
  goes with the device smoke [025].
- The PNG is the full screen (1080 x 2400 on the emulator, 4.4 MB: the CRT scanlines
  compress badly). If the sheet opens slowly on a phone, halve it before the encode.
