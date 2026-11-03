# [104] A wall skin can change a seeded run

**Status:** open
**Milestone:** M5
**Depends on:** [078], [103]
**Created:** 2026-11-03
**Closed:** -

## Problem
Skins should be cosmetic ([078]): a seed plays the same with any skin. Found in [103]:
the store scene with `RIX_SEED=1 RIX_LEVEL=9 RIX_SHOT_LINES=1 RIX_SHOT_SIZE=1080x2160`
claims two parts with the PLAIN wall (state hash `b8e327549cb3db72` at tick 160), but
with `RIX_SKINS=brick` or `circuit` the player dies on the second line. The line skins
(`chain`, `laser`) do not change it.

Other runs keep their hash with a wall skin: the five replay seeds (`brick` and
`circuit`, no renderer), `just shot` seed 1 at the default size and at 1080x2160, the
same store run at 1206x2622, and seeds 4 (level 6) and 9 (level 9) of line set 1 at both
store sizes. So the difference is rare.

Why it matters: seeds are shared (decisions, gate answer 4); two players with different
wall skins can get different enemy paths from one seed. No player-facing replay exists,
so the effect is small.

## Approach
Find what a wall skin changes outside the look. The PLAIN wall gets `Mesh2d` and
`MeshMaterial2d`, the others a tiled `Sprite` (`skins::insert_wall_look`). Not checked:
the walls then sit in other archetypes, so physics may visit contacts in another order,
and a near tie (an enemy that touches two bodies in one step) can end another way. Log
the enemy positions per tick in both runs and find the first tick that differs.

## Verification
The run above gives the same `RIX_STATE` (or the same death) with `RIX_SKINS=` empty,
`brick` and `circuit`; `just replay` passes.

## Notes
