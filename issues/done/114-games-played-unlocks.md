# [114] Unlocks for games played

**Status:** done
**Milestone:** M8
**Depends on:** [115], [116]
**Created:** 2026-11-05
**Closed:** 2026-11-05

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
- Done before [117] (2026-11-05): this issue makes the unlock model, so [117] adds its two
  skins as `Games(50)` and `Games(1000)` and does not ship them free or by level.
- `Stats.games: Option<u64>` (`None` in an old save: `Stats::games` gives its runs);
  `record_run` adds 1 only for a run with a claim. `Unlock` (`Free`, `Level`, `Games`) on
  every skin, `all_unlocks` lists them part by part. The tiers: SCANLINES 5, SHOCKWAVE 25,
  GRID 100, FIREWORK 250, STARS 500 (the wall at 50 and GOLD at 1000 come with [117]).
- The game over screen shows "NEW SKIN: ...\nPICK IT IN SKINS" above "YOU DIED!".
- The skins screen: LINE, WALLS, FILL and KILL buttons; the preview has the fill between
  the wall and the line; a KILL step plays the kill there; one row per part with its next
  locked skin ("GRID FILL: 30 / 100 GAMES"), then "BEST LEVEL n   GAMES n". The rows that
  said UNLOCKED are gone: the buttons step through the unlocked skins.
- `effects::pixel_explosion` spawns the pixel burst (the kill and the preview).
- Evidence: `cargo test` 155 passed (the count rule and an old save, the tiers, the locked
  label, the part steps, the rows and their fit); clippy clean with and without
  `dev_tools`; `just replay` unchanged; `just shot` md5 e170997305a806320c6f9f967cf3cf5b;
  a `skins` shot (best level 12, 30 games) and a game over shot (games 4 to 5: SCANLINES
  FILL), looked at.
