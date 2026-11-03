# [091] Locked MUSIC button: gray, with a diagonal line across it

**Status:** done
**Milestone:** M4
**Depends on:** [089]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Pedro (D27): while the music is locked, the button "shouldn't say \"Music: Level 3\" it should
be greyed out and with a sort of diagonal line (think of a prohibited sign)".

## Approach
- While `Save::soundtrack_open` is false, the MUSIC row reads "MUSIC", its box and text
  are gray, and a gray line runs across the box from the bottom left to the top right
  (the bar of a no sign). A tap still does nothing.
- `widget` gets what it needs to draw a button gray (a small helper or a color
  argument); the settings screen and the pause panel both show it.

## Verification
A test: a locked MUSIC row has the gray look and the line, and an open one has neither;
`cargo test`; an emulator screenshot of the settings.

## Notes
- Done 2026-11-03. `widget::locked_button`: the `Locked` marker on a button drawn with a dark
  gray fill and a lighter gray (`LOCKED_GRAY`) frame and text; the `cross_out` observer adds
  the line from corner to corner, 8 units inside. `Setting::locked` picks it for MUSIC.
  The first gray (`BASE_GRAY` for all) was too dark to read the word.
- `RIX_SHOT_SCENE=settings` (new) shows the settings screen in a shot.
- Evidence: tests `a_locked_button_gets_a_line_from_corner_to_corner`,
  `only_a_locked_music_row_is_a_locked_button`, `music_is_locked_until_the_soundtrack_opens_then_starts_off`;
  the settings shot; on the emulator the locked MUSIC button, and a tap leaves it so.
