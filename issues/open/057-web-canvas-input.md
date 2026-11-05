# [057] Web canvas and input

**Status:** open
**Milestone:** M6
**Depends on:** [019]
**Created:** 2026-11-02
**Closed:** -

## Problem
The window is fixed at 625x625 and not resizable (`base.rs`). On itch.io the game runs in
a browser canvas (D13), on desktop and phone browsers, so it must fit and resize with the
page and go fullscreen.

## Approach
For `wasm32`: the canvas fits its parent (`fit_canvas_to_parent`) on top of the logical
playfield ([019]); a fullscreen button; Esc and losing focus pause ([034]); mouse and
touch as today; no gamepad. The native desktop window stays a dev tool (625x625).

## Verification
In a desktop browser and a phone browser: resize, go fullscreen and play a run; the
playfield stays letterboxed and input stays aligned.

## Notes
- D9 (2026-11-02): Steam and GOG are dropped; this issue now serves itch.io only.
- D13 (2026-11-02): itch.io gets the web build only, so this issue is about the web canvas.
- [124]: the daily challenge reads the date with `web_time::SystemTime` (std's panics on
  the web). Check the CHALLENGES screen's daily row in the web build.
