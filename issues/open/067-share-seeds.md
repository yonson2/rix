# [067] Show, share and play a seed

**Status:** open
**Milestone:** M4
**Depends on:** [019], [026], [036], [038]
**Created:** 2026-11-02
**Closed:** -

## Problem
[012] Q4: v1 shows and shares seeds. Each run gets a new seed (no daily seed). Today the
seed lives only in `LevelInfo.seed`; a player cannot see it, send it, or play a seed that
someone sent.

## Approach
Show the run seed on the game over screen ([038]) and in the pause menu ([034]), as a short
code (for example base32 of the `u64`). Share it with the OS share sheet on iOS and
Android, and copy it to the clipboard on web and desktop. A "Play seed" entry in the menu
([036]) takes a typed or pasted code and starts a run with it. The same seed must give the
same game on every device: that needs the fixed playfield ([019]) and a fixed system order
([026]).

## Verification
A unit test round-trips seed <-> code. `RIX_SEED=<n>` and "Play seed" with the code of
`<n>` give the same first-level shot. The game over screen shows the code.

## Notes
- From [012] Q4 (Pedro, 2026-11-02).
- [072] (2026-11-03): a seed now gives the same state at any frame rate (`just replay`); a
  check across targets is [075].
