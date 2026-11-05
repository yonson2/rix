# [122] Countdown challenges

**Status:** done
**Milestone:** M9
**Depends on:** [109], [121]
**Created:** 2026-11-05
**Closed:** 2026-11-05

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
- Result: `src/game/countdown.rs`. `new_run` inserts a `Countdown` for a rule with a
  countdown (`Rules::countdown`, `COUNTDOWNS`) and removes it otherwise. `tick_clock`
  runs in `FixedUpdate` (`TickTimers`, `PausableSystems`), so a pause, a level transition
  and a death stop it; the Time Stop stops only the physics clock, not this one. At 0 it
  enters `IsPaused::Dying`: the death's freeze, then the game over screen, whose title is
  "TIME!" in yellow (`game_over::title`). The clock sits under the top wall (inner top
  - 40, clear of the tips). In the last 10 s it grows to 1.5, beats with each second,
  turns red and ticks; at 0 a big "TIME!" pops and flashes white and red. The `time_up`
  buzzer plays in place of the death sound. The tick also gives a light haptic pulse.
- Sounds: `tick` (take `tick-3`, peak -8 dB, 0.05 s) and `time_up` (take `time_up-1`,
  peak -4 dB, 1.2 s), ElevenLabs, picked by measurement; rows in `audio-src/sources.md`.
- Dev: `RIX_CLOCK=<secs>` gives a run a countdown (fractions allowed, to shoot the flash).
- Evidence: `cargo test` (185 passed; new: `the_clock_shows_minutes_and_seconds_rounded_up`,
  `the_last_seconds_grow_beat_and_turn_red`, `the_clock_ends_the_run_at_0_only`,
  `a_death_is_not_time_up`, `the_title_says_time_only_when_the_clock_ran_out`, the
  countdown asserts in the rules test). `just replay` passes with the countdown seed
  2010-0000-003 (`451a0a24dea5524f`). Shots (`RIX_CLOCK`, in `target/shots/`): `clock-hud.png`
  shows "1:12" in white in the HUD; `clock.png` a big red "0:09"; `RIX_CLOCK=2.6` shows the "TIME!" flash
  over the frozen field; the game over shot says "TIME!". Default `just shot` unchanged
  (md5 e170997305a806320c6f9f967cf3cf5b). fmt, both clippy runs, `cargo build` clean.
