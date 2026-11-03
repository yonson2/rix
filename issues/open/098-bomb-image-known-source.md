# [098] Bomb image with a known source

**Status:** open
**Milestone:** M5
**Depends on:** [011]
**Created:** 2026-11-03
**Closed:** -

## Problem
`assets/items/bomb_basic.png` (100x100, a blue cartoon bomb with a fuse) has no known
source ([011] Q5). AGENTS.md: an asset with an unknown licence does not ship. The bomb
item and the wiki row draw it (`basic_bomb::look`, `GlowMaterial` over the texture).

## Approach
Draw a new 100x100 bomb with a tracked script (PIL, the game palette), same size and
anchor, so `basic_bomb.rs` does not change. Record it in `CREDITS.md`.

## Verification
`git log` shows the new PNG made by the tracked script; a `just shot` with a bomb (or the
wiki shot `RIX_SHOT_SCENE=wiki` with the bomb met) shows it; `cargo test` passes.

## Notes
- Found in [052] (2026-11-03) while listing every shipped asset for the credits.
