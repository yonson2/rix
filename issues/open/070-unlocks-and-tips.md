# [070] A tip on first sight of each kind

**Status:** open
**Milestone:** M4
**Depends on:** [032], [044], [045], [046], [048], [049]
**Created:** 2026-11-02
**Closed:** -

## Problem
Pedro's idea (2026-11-02, D18): once all enemies and items exist and the game is about
ready, a new player should not meet everything at once. Not every enemy and item is
unlocked at the start, and the first time an element of a kind spawns, a short tip
explains it.

## Approach
D23 (2026-11-03): tips only, no locks. The level gates inside a run stay as they are
(spawn table [032], `ITEM_LEVELS` [046]). A list of the kinds the player has met lives in
the save ([048]). The first time a kind spawns and the player has not met it, a short tip
names it and says what it does, with the element marked (reuse the overlay pattern of
[049]); the kind goes into the list. Nothing is locked, so a shared seed gives the same
game to everyone ([067]). The Star's tip comes with [069].

## Verification
A fresh save: the first spawn of each kind shows its tip once; a second run does not show
it again (a test on the met list and a scripted run with a fresh `RIX_SAVE`); a seed replays
the same state hash with tips on.

## Notes
- Overlaps [049] (the rules tutorial): [049] teaches the line and the claim; this issue
  teaches each enemy and item.
- From [049] (2026-11-03): the tutorial is `src/game/tutorial.rs` (overlay with one rule
  text, a ghost finger and SKIP; a flag in `Save`). A tip here can reuse its overlay
  pattern; [049] does not pause the game, a tip that does needs its own rule.
- 2026-11-03: Pedro chose tips only (D23); the lock rule and the seed question are gone.
