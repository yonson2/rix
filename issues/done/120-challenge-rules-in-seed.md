# [120] Challenge rules in the seed code

**Status:** done
**Milestone:** M9
**Depends on:** [067], [109]
**Created:** 2026-11-05
**Closed:** 2026-11-05

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
- 2026-11-05 result: `src/game/rules.rs`. `Rules` holds each value as an index (0: the
  normal game) in the proposed layout: version 1 in bits 0-1 over the 40-bit field, kinds
  2-6, speed 7-8, count 9-10, countdown 11-13 (`COUNTDOWNS`), target 14-15, items 16-17,
  start level 18-21; 22 and 23 spare. `Rules::of_seed` gives `None` for version 0 with
  rules, versions 2 and 3, countdown 6 and 7, items 2 and 3, or a spare bit; `Rules::seed`
  puts rules over a field and leaves a field as it is for the normal rules. `new_run`
  inserts the seed's `Rules` (unknown rules: the normal ones); PLAY SEED does not play an
  unknown challenge and says "UNKNOWN CHALLENGE" under the code. The tables of the effects
  (speeds, counts, targets) come with [121]; `Rules::seed` has a `dead_code` expectation
  until the CHALLENGES list uses it ([123]).
- `GameRng` takes the whole seed, so a challenge's field differs from the plain field seed's
  (a challenge code is its own field, as [123] wants).
- Evidence: `cargo test` 172 passed (`every_40_bit_seed_plays_the_normal_rules`,
  `rules_round_trip_through_the_seed_and_its_code`,
  `a_code_with_an_unknown_version_or_value_has_no_rules`,
  `a_challenge_code_this_build_does_not_know_does_not_play`); clippy clean both ways;
  `just replay` same hashes; `just shot` md5 `e170997305a806320c6f9f967cf3cf5b`.
