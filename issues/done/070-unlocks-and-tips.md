# [070] A tip on first sight of each kind

**Status:** done
**Milestone:** M4
**Depends on:** [032], [044], [045], [046], [048], [049]
**Created:** 2026-11-02
**Closed:** 2026-11-03

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
- Done 2026-11-03 (in `src/game/tutorial.rs`, beside the [049] tutorial it waits for).
  After the tutorial, the first enemy or item on the field whose kind is not in `Save::met`
  gets a tip for 3.5 s: two lines (name and what it does; items: "TOUCH THE GEM:" and the
  effect, with the real `BOOST_FACTOR`, `BOOST_SECS` and `STOP_SECS`) and a white ring that
  follows the element. Then the kind goes into `met`. One tip at a time; a pause, a death or
  a level clear cancels it (not met), so it shows again later. The game never stops for a
  tip, so a seed replays the same. The texts sit next to the kinds (`EnemyKind::name`/
  `about`, `Item::name`/`about`) for the wiki ([071]).
- My choices (Pedro can change them): no pause during a tip; the text sits where the
  tutorial's text sits (it can cover the element near the top; the ring still marks it);
  the bullets of the Shooter get no tip of their own.
- Evidence: `cargo test` 103 passed (every kind has a two-line tip of at most 24 characters
  and its own id; a new kind gets one tip after the tutorial, then counts as met, and a
  second one of that kind gets none; a pause cancels a tip and it shows again); clippy clean
  with and without `dev_tools`; `just replay` unchanged; `just shot` md5 unchanged. Scripted
  runs (seed 2): a save with the tutorial done and nothing met shows "SQUARE / IT FADES AND
  JUMPS AWAY" with the ring on the square; the same save with `"met": ["Square"]` gives the
  baseline picture (md5 e170997305a806320c6f9f967cf3cf5b). With tips on (fresh save), the
  seed-1 replay hash is the baseline (7f4efff7fa02493e).
