# [111] Medals and the RECORDS screen

**Status:** open
**Milestone:** M7
**Depends on:** [048], [070]
**Created:** 2026-11-05
**Closed:** -

## Problem
D39: medals come in 1.1, with the leaderboards. Feats that only Rix has give goals past
the best score.

## Approach
- The medals as data (id, name, rule text). `Save.medals` holds the earned ids, as `met`
  does. A system checks each feat at its moment; a new medal shows a short notice, as a
  skin unlock does, once.
- A RECORDS button on the title opens the RECORDS screen: every medal, earned or not, with
  its rule text. [113] adds the lifetime totals to the same screen.
- No medal in the first run (D27: the first run unlocks nothing). Challenge runs: [109]
  Q3.
- The first list:
  - ONE STROKE: clear a level with one claim.
  - FEAST: eat 3 enemies with one claim.
  - PACIFIST: clear a level with no kills.
  - STAR STRIKE: kill 5 enemies during one Star.
  - BOMB SQUAD: kill 3 enemies with one bomb.
  - GREEDY: clear a level at 90 % or more.
  - KNOW YOUR ENEMIES: meet every enemy kind.
  - DEEP DIVE: reach level 25.

## Verification
`cargo test` passes with a test per medal (earned at its feat, once, saved); a `records`
shot scene (`RIX_SHOT_SCENE=records`) shows the screen with earned and open medals.

## Notes
- From the 2026-11-05 brainstorm (D39: "ok" to medals in 1.1).
- [112] makes each medal an achievement on Game Center and Play Games.
