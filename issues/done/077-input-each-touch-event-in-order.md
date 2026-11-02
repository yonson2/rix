# [077] Input handles each touch event in order

**Status:** done
**Milestone:** M3
**Depends on:** [072], [074]
**Created:** 2026-11-03
**Closed:** 2026-11-03

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
- Done 2026-11-03:
  - `handle_input_events` reads `WindowEvent` (bevy_winit writes it in arrival order, with
    touches, clicks and cursor moves in one stream) and `InputState::record` turns each
    event into a `Pointer` step (`Press(p)`, `Move(p)`, `Release`) of the one pointer.
    `handle_input` runs the old line logic once per step. It still sends one
    `InputPosition` per tick (the latest position) before the steps, so the arrow and the
    shots stay the same. `reset_input_state_flags`, the `just_*` flags and the unused
    `last_position_screen` are gone.
  - Found on the way: with several steps in one tick, a line could send `DrawLine` and
    then `ClearLines` in one tick. `handle_draw_line` spawns through commands, so
    `handle_clear_lines` cannot see that segment, and it stays: on the emulator an enemy
    hit the stale segment after a claim and the run ended. `handle_input` now merges a
    tick's line messages: a clear drops the segments drawn before it in that tick, then
    one `ClearLines` and the rest of the `DrawLine`s go out.
  - The pause also clears pending `WindowEvent`s (with `MouseButtonInput` and
    `TouchInput`), so an old tap does not start a line. bevy_picking reads `WindowEvent`
    too; Rix uses no picking yet.
  - The shot harness (`drive_line`) writes each touch as `TouchInput` and as
    `WindowEvent::TouchInput`, as winit does. With only `WindowEvent`, bevy_picking
    spawned a touch pointer on the start and never despawned it (it despawns on the
    `TouchInput` end), which showed as one extra entity in the census.
- Verification 2026-11-03: `cargo test` 44 passed, among them
  `a_flick_with_press_moves_and_release_in_one_tick_claims` (one `BuildPolygon`, no
  `DrawLine` left; the old code started at the far wall and claimed nothing) and
  `a_press_and_moves_in_one_tick_start_at_the_wall`. Clippy clean with and without
  `dev_tools`; fmt clean. `just shot` md5 unchanged for seeds 1, 7, 10; `just replay`
  passes. `just shot-flow`: the same census, plus the touch pointer entity of
  bevy_picking while the scripted touch is down. Android emulator (debug APK):
  `input motionevent DOWN, MOVE, MOVE, UP` with no pauses claims (35 %, then 10 %), and
  after the second flick the arrow sat on the claim edge for 8 s with no death.
