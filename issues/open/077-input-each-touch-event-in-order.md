# [077] Input handles each touch event in order

**Status:** open
**Milestone:** M3
**Depends on:** [072], [074]
**Created:** 2026-11-03
**Closed:** -

## Problem
`handle_input_events` (`src/game/user_input.rs`) reads all touch and mouse events of one
tick into one `InputState`: the last position, one `just_pressed` and one
`just_released` flag. `handle_input` then sees one point per tick. So:
- A release in the same tick as the last move cancels the line: the move that reaches the
  wall is never used. A quick flick that ends on the wall and lifts within one tick
  (15.6 ms at 64 Hz, [072]) loses its line.
- A press and its first moves in one tick start at the last position, which is in the
  field, so no line starts.
- The points between are lost, so a fast curve becomes one straight segment.
Seen on the Android emulator (2026-11-03, about 10 frames per second): `adb shell input
swipe` from the wall gives no line, and `input motionevent DOWN, MOVE, MOVE, UP` with no
pauses starts a line but cancels it. With 0.3 s pauses between the events, the same line
claims 46 %. The logic was the same in `Update` before [072]; ticks at 64 Hz on a 120 Hz
phone merge two frames.

## Approach
Turn each event into a pointer step (press, move to p, release) in order, and run the path
logic once per step, not once per tick. Keep one pointer (the first touch, or the mouse).

## Verification
A unit test feeds press, moves and release in one tick (wall, field, far wall, release)
and gets one `BuildPolygon`; another feeds press plus moves in one tick and starts the
line at the wall. `cargo test`, `just shot` and `just replay` pass. On the emulator, the
`motionevent` sequence with no pauses claims.

## Notes
- Found during the [072] Android emulator check, 2026-11-03.
