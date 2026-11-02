# [030] Kill count counted twice; kills and explosions on state exit

**Status:** done
**Milestone:** M3
**Depends on:** [026]
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
Read from the code (2026-11-02), not yet reproduced:
- `on_despawn_enemy` (`enemies/mod.rs`) adds 1 to `kill_count`, then the `OnRemove, Enemy`
  observer `on_remove_enemy` adds 1 again: an eaten enemy counts twice.
- `on_remove_enemy` runs for every Enemy removal, including the `despawn_screen::<RixComponent>`
  cleanup on leaving `InGame` (level clear and death). So every enemy alive at the end of a
  level counts as a kill and spawns a pixel explosion, and the game-over "Enemies Killed"
  includes the enemies that killed you.

## Approach
Count kills and spawn the death effect where the kill happens (the eat and the bomb
paths), not on component removal. Decide with [039] what a level-clear does to living
enemies.

## Verification
A test (headless app or system test) eats one enemy and expects `kill_count == 1`; ending
the run with 3 living enemies leaves the count unchanged.

## Notes
- Done (2026-11-02): both bugs were real. `on_remove_enemy` (the `Remove<Enemy>` observer)
  is gone; `on_despawn_enemy`, the observer of the kill trigger (eat and bomb), now counts
  the kill once and spawns the pixel explosion. Living enemies at a level clear or a death
  despawn with `DespawnOnExit(Phase::Playing)`: no count, no explosion. What a level clear
  shows for them stays with [039].
- Evidence: test `an_eaten_enemy_counts_once_and_a_state_exit_counts_none` (one kill: count
  1 and one explosion; then 3 enemies despawned: still 1 and 1). `cargo test` 35 passed,
  clippy clean both ways, fmt clean. Shots unchanged (seed 1 `109eb09e`, 7 `197a00c8`,
  10 `78d871fa`). `just shot-flow`: every census is 1 lower (the observer entity is gone);
  at the level clear 1 more lower and at game over 2 more lower: the explosions of the
  enemies that left with the state are gone.
