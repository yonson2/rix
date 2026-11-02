# [057] Desktop window and input

**Status:** open
**Milestone:** M6
**Depends on:** [019]
**Created:** 2026-11-02
**Closed:** -

## Problem
The desktop window is fixed at 625x625 and not resizable (`base.rs`); no fullscreen, no
keyboard shortcuts, no gamepad. Steam players expect a resizable window and fullscreen;
Steam Deck needs gamepad or touch.

## Approach
Resizable window and fullscreen toggle on top of the logical playfield ([019]); Esc pauses
([034]); mouse as today; gamepad drawing (a cursor moved by the stick) only if [010] or
the Steam Deck plan asks for it.

## Verification
On macOS: resize, toggle fullscreen and play a run; the playfield stays letterboxed and
input stays aligned.

## Notes
