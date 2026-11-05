# [114] Unlocks for games played

**Status:** open
**Milestone:** M8
**Depends on:** [115], [116], [117]
**Created:** 2026-11-05
**Closed:** -

## Problem
D39 (Pedro's idea): looks unlock at 5, 25, 50, 100, 250, 500 and 1000 games. Unlocks by
best level reward skill; these reward time spent.

## Approach
- `Stats.games`: the runs with at least one claim (D39), so quick deaths do not count. A
  challenge run counts too ([109] Q3). An old save starts with its `runs`.
- The tiers (D39, the agent's pick; the names are placeholders for [115] to [117]):
  5 SCANLINES fill, 25 PIXEL kill effect, 50 the new wall, 100 GRID fill, 250 the second
  kill effect, 500 STARS fill, 1000 GOLD line.
- A new unlock shows a notice on the game over screen. The skins screen gets the FILL and
  KILL parts, and a locked look says "18 / 25 GAMES". `Skins` gets the new fields with
  serde defaults, so old saves load.

## Verification
`cargo test` passes with tests for the count rule, the tiers and the locked label; a
`skins` shot shows the new parts and labels.

## Notes
- From the 2026-11-05 brainstorm (D39: "1000 yeah", "ok" to the claim rule).
