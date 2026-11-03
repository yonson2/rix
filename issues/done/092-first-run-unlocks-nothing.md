# [092] The first run unlocks nothing

**Status:** done
**Milestone:** M4
**Depends on:** [078], [089]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Pedro (D27): "Tutorial levels shouldn't be able to unlock stuff." Answer: the whole first
run (the run with the first-run tutorial) unlocks nothing.

## Approach
- `skins::record_level` does nothing while `Save::stats.runs` is 0 (no run finished yet):
  no best level, no notice. The best level counts from the second run.
- Tips and the wiki's `met` list stay as they are (they teach, they are not rewards).

## Verification
A test: a level start in the first run leaves `best_level` at 0 and shows no notice; after
one finished run it records the level; `cargo test`.

## Notes
- A first run left from the pause menu (MENU) records no run, so the next run is still the
  first one.
- Done 2026-11-03. `record_level` returns at once while `stats.runs` is 0.
- Evidence: test `the_first_run_unlocks_nothing` (a level 5 start in the first run leaves
  the best level at 0 and shows no notice; after one finished run it records 5); the
  notice test now starts from a save with one run.
