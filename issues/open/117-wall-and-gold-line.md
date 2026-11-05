# [117] Two more skins: a wall and the GOLD line

**Status:** open
**Milestone:** M8
**Depends on:** [078]
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
