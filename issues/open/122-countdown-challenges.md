# [122] Countdown challenges

**Status:** open
**Milestone:** M9
**Depends on:** [109], [121]
**Created:** 2026-11-05
**Closed:** -

## Problem
D39 (Pedro): "countdown mode (highest score in n time)".

## Approach
- A run clock (in `FixedUpdate`, pausable) counts down from the rule's time. The HUD shows
  it. At 0 the run ends, and the game over screen says "TIME!".
- A death: [109] Q2 (proposal: it costs 10 seconds, the line goes, and the level goes on).
- The Time Stop item stops the enemies, not the clock.

## Verification
`cargo test` passes with tests for the clock, its end and the death rule; `just replay`
passes with a countdown seed; `just shot` shows the clock in the HUD.

## Notes
- From Pedro's message (2026-11-05, D39).
