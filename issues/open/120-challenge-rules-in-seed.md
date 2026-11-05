# [120] Challenge rules in the seed code

**Status:** open
**Milestone:** M9
**Depends on:** [067], [109]
**Created:** 2026-11-05
**Closed:** -

## Problem
D39 (Pedro): a challenge is a special seed. The code carries the rules, so a tap in the
challenge list or the code in PLAY SEED gives the same challenge to every player, as D23
wants.

## Approach
- A run's seed is a `u64`, and a new run uses 40 bits (8 characters). The high 24 bits
  hold the rules. Rules of 0 are the normal game, so every code so far keeps its game. A
  challenge code has up to 13 characters (at most `FZZZ-ZZZZ-ZZZZ-Z`).
- The rules (a proposal, 22 bits and 2 spare): version (2 bits); enemy kinds (5 bits, one
  per kind; none set means the normal table); enemy speed (2 bits: 1x, 1.25x, 1.5x, 2x);
  enemy count (2 bits: 1x, 1.5x, 2x, 3x); countdown (3 bits: off, 60, 90, 120, 180 or
  300 s); clear target (2 bits: 75, 60, 85 or 90 %); items (2 bits: normal or none);
  start level (4 bits: 1 to 16).
- The points factor follows from the rules (harder rules pay more). The code does not
  carry it, so a code cannot ask for more points with easy rules.
- `new_run` puts a `Rules` resource from the seed. A code with an unknown version or value
  does not play: PLAY SEED says "UNKNOWN CHALLENGE".
- Pure functions with tests (rules to bits and back; every old code gives the normal
  rules). This issue adds no rule effect ([121]).

## Verification
`cargo test` passes with the round-trip tests; `just replay` passes unchanged.

## Notes
- From Pedro's message (2026-11-05, D39).
- 2026-11-05: [109] answered; nothing here changes.
