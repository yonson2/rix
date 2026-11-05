# [117] Two more skins: a wall and the GOLD line

**Status:** open
**Milestone:** M8
**Depends on:** [078], [114]
**Created:** 2026-11-05
**Closed:** -

## Problem
The unlock tiers ([114]) need a new wall (50 games) and a top reward (1000 games).

## Approach
- One wall skin, drawn in code as the brick and circuit textures are (`src/game/skins.rs`).
- The GOLD line: a glowing gold line, for 1000 games.
- Look only ([067]).

## Verification
`cargo test` passes; `just shot` with each skin (`RIX_SKINS`) shows it; look at the PNGs.

## Notes
- From the 2026-11-05 brainstorm (D39).
- 2026-11-05: [114] came first; add the wall as `Unlock::Games(50)` and GOLD as `Unlock::Games(1000)` (`src/game/skins.rs`), with a test in `looks_unlock_at_5_25_100_250_and_500_games`.
