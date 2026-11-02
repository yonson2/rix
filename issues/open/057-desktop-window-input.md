# [057] Desktop window and input

**Status:** open
**Milestone:** M6
**Depends on:** [019]
**Created:** 2026-11-02
**Closed:** -

## Problem
The desktop window is fixed at 625x625 and not resizable (`base.rs`); no fullscreen, no
keyboard shortcuts, no gamepad. Players on itch.io (web and desktop) expect a resizable
window and fullscreen.

## Approach
Resizable window and fullscreen toggle on top of the logical playfield ([019]); Esc pauses
([034]); mouse as today; no gamepad (no Steam, D9).

## Verification
On macOS: resize, toggle fullscreen and play a run; the playfield stays letterboxed and
input stays aligned.

## Notes
- D9 (2026-11-02): Steam and GOG are dropped; this issue now serves itch.io only.
