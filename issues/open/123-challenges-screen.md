# [123] The CHALLENGES screen

**Status:** open
**Milestone:** M9
**Depends on:** [109], [114], [121], [122]
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
- `Save` keeps the best per challenge code. A challenge run counts toward the lifetime
  stats, the games played and the medals, not the best level, the best score or the main
  board ([109] Q3).
- Some challenges open with games played (`Stats.games`, [114]; [109] Q4). A locked entry
  says "OPENS AT 20 GAMES (12 / 20)". A shared or typed code always plays. The RECORDS
  screen's next unlock ([113]) lists the next challenge too. The schedule (the agent's
  pick, between the look tiers of [114]): open at the start: 60 SECONDS, BALL PIT, BARE
  HANDS; 10 games: OVERDRIVE; 15: SWARM; 20: BULLET HELL; 30: SPLIT DECISION; 40: THREE
  MINUTES; 60: GREEDY; 75: DEEP END.
- The first list ([109] Q6: it stays):
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
