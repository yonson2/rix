# [119] Secret looks

**Status:** done
**Milestone:** M8
**Depends on:** [114], [115]
**Created:** 2026-11-05
**Closed:** 2026-11-05

## Problem
Hidden goals surprise players, and players talk about them.

## Approach
- Three looks with hidden conditions. The skins screen shows "???" until the player finds
  one; then a notice says "SECRET FOUND".
- The conditions: clear a level at exactly 75.0 % (one decimal); draw one line that
  touches all four walls; end a run on a palindrome score of 1,000 or more (for example
  12,321).
- The looks: one line, one wall and one fill, made in this issue.

## Verification
`cargo test` passes with a test for each condition; `just shot` shows each look.

## Notes
- From the 2026-11-05 brainstorm (D39).
- 2026-11-05 result: `src/game/secrets.rs` holds the three `Secret` feats and their pure
  tests: `exact_clear` (rounds the claimed fraction to 75.0 % at `OnEnter(Phase::
  LevelTransition)`), `touches_four_walls` (a `BuildPolygon` line within 6 units of each
  outer wall: from a corner to the far corner, or past a wall on the way; a line ends only
  1 unit from the boundary, so a pass is possible) and `palindrome` (a final score of
  1,000 or more, checked at the game over before `record_run`). `find` keeps each in
  `Save::secrets` once, not in the first run (D27, as medals). The notice is three lines,
  "SECRET FOUND / HONEYCOMB FILL / PICK IT IN SKINS": in one line "HONEYCOMB FILL" made it
  382 units wide, more than the 380 between the walls.
- The looks (`Unlock::Secret`): HONEYCOMB fill (75.0 %; hex cells outlined 2 units wide, a
  21 x 36 tile with smooth edges), CHECKER walls (four walls; 16 unit squares: 8 unit ones
  split into color fringes under the CRT pass) and the VOLT line (palindrome; a blue glow
  with a white arc that jumps every 0.06 s; each joint is a kink on alternate sides, as a
  hash of the joint and the time slot: random offsets read as a soft wave).
- The skins screen shows "SECRETS: ??? ??? ???" (a found look shows its name) under the
  lock rows, which no longer list the secret looks; the rows moved up 4 and closer by 2.
- Evidence: `cargo test` 168 passed, with the secrets tests
  (`a_clear_at_75_0_percent_is_exact`, `..._finds_its_secret_with_a_notice`,
  `a_line_touches_four_walls_from_corner_to_corner_or_on_its_way`,
  `a_palindrome_score_has_four_digits_or_more`, `a_secret_is_found_once_and_not_in_the_
  first_run`) and `a_run_that_ends_on_a_palindrome_finds_its_secret` (game over);
  clippy clean with and without `dev_tools`; `just replay` same hashes; `just shot` md5
  `e170997305a806320c6f9f967cf3cf5b` as before; `RIX_SKINS=volt,checker,honeycomb just
  shot`, the `store` scene and the `skins` scene show each look.
