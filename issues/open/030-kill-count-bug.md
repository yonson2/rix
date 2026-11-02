# [030] Kill count counted twice; kills and explosions on state exit

**Status:** open
**Milestone:** M3
**Depends on:** [026]
**Created:** 2026-11-02
**Closed:** -

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
