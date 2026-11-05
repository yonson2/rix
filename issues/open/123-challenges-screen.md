# [123] The CHALLENGES screen

**Status:** open
**Milestone:** M9
**Depends on:** [109], [121], [122]
**Created:** 2026-11-05
**Closed:** -

## Problem
D39 (Pedro): a challenge section with a list of special seeds. A tap plays one, and its
code also plays from PLAY SEED.

## Approach
- A CHALLENGES button on the title opens a list: each curated challenge with its name, a
  one-line rule text and the local best. A tap plays it.
- Each curated challenge is a fixed code (field seed and rules), so every player gets the
  same field.
- PLAY SEED takes challenge codes and shows the challenge name, or "CUSTOM CHALLENGE" for
  a code that is not on the list.
- The game over screen shows the challenge name and its best. The share text gives the
  name and the code.
- `Save` keeps the best per challenge code. What a challenge run counts toward: [109] Q3.
- The first list (a proposal, [109] Q6):
  - BALL PIT: balls only, 2x enemies.
  - BULLET HELL: shooters only.
  - SPLIT DECISION: splitters only.
  - SWARM: 3x enemies.
  - OVERDRIVE: 1.5x speed, more points.
  - 60 SECONDS: the most points in one minute.
  - THREE MINUTES: the most points in three minutes.
  - GREEDY: 90 % to clear a level.
  - BARE HANDS: no items.
  - DEEP END: start at level 10.

## Verification
`cargo test` passes (every listed code parses to its rules; the names); a `challenges` shot
scene shows the list; `just shot-flow` style run: a tap starts the challenge.

## Notes
- From Pedro's message (2026-11-05, D39).
