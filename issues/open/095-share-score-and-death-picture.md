# [095] Share: the score and a picture of the death

**Status:** open
**Milestone:** M4
**Depends on:** [067], [038]
**Created:** 2026-11-03
**Closed:** -

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
-
