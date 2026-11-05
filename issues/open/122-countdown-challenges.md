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
- A death ends the run, as in the normal game ([109] Q2).
- The countdown effect (Pedro: "a cool countdown effect"): in the last 10 seconds the clock
  grows, pulses and turns red, with a tick each second (a new sound, `audio-src/sources.md`);
  at 0 a "TIME!" flash and a buzzer before the game over screen.
- The Time Stop item stops the enemies, not the clock.

## Verification
`cargo test` passes with tests for the clock, its end and the death rule;
`just shot` at the last seconds shows the effect; `just replay`
passes with a countdown seed; `just shot` shows the clock in the HUD.

## Notes
- From Pedro's message (2026-11-05, D39).
