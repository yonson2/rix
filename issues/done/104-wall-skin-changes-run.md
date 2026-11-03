# [104] A wall skin can change a seeded run

**Status:** done
**Milestone:** M5
**Depends on:** [078], [103]
**Created:** 2026-11-03
**Closed:** 2026-11-03

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
- 2026-11-03, cause: a per-tick trace of the enemies (temporary, in `log_state`) showed
  both runs equal up to tick 96. At tick 97 the first claim catches a bomb, and
  `basic_bomb::activate` drew the same index from `GameRng` in both runs, but the enemy
  list came in query order: `[660, 668, ...]` with PLAIN walls, `[668, 660, ...]` with
  brick. The enemies sit in different archetypes (each kind has its own components), and
  Bevy does not visit them by archetype id (660 in archetype 498 came before 668 in 496).
  The wall skin's own archetypes (a `Sprite`, not `Mesh2d`) changed that order. Physics
  was equal: the positions matched until the bomb.
- Fix: `setup::by_place` (x, then y, `total_cmp`). The bomb sorts its candidates with it;
  `splitter::wander` (one draw per splitter whose turn ends, two can end in one tick) and
  `ball::add_chaos_on_collision` (one draw per ball hit, in the order avian sends the
  collisions) sort too. Not changed: the kill's explosion seed (cosmetic, one draw per
  kill), `square` teleports and `hatch` (all draw for alike entities, so the order gives
  the same set), `level::start_transition` (cosmetic fade direction, and the next level
  reseeds). Entity ids are no stable key: cosmetic spawns change them.
- AGENTS.md hard rule: draws for several entities go in `by_place` order.
- Checks: new test `the_victim_does_not_hang_on_the_archetype_order` failed before the
  fix (victim x -50 vs 50) and passes; `by_place_orders_by_x_then_y`; 133 tests; clippy
  (with and without `dev_tools`), fmt. The run above: `b8e327549cb3db72` with no skin,
  `brick`, `circuit`, `chain` and `laser`. `just replay` keeps its hashes; `just shot` md5
  `e170997305a806320c6f9f967cf3cf5b` (unchanged). `just store-art`: five game shots
  changed only in the sparks (the bounce bursts come in the new order, so `FxRng` draws
  land on other sparks); scores, enemies and claims are the same.
