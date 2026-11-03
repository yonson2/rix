# [096] The Star's line kills the enemies that touch it

**Status:** done
**Milestone:** M4
**Depends on:** [069]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
D28 (Pedro): while the Star runs, an enemy that crosses the line should be destroyed.
Today `handle_die` ignores the touch and the enemy passes through the line.

## Approach
`enemies::handle_enemy_collision`: an enemy that touches the line triggers
`DespawnEnemy` (the claim's kill: points, kill count, sound, explosion) while `Star`
exists, and `Die` otherwise; each enemy once per frame. The Star's text says it.

## Verification
A test: a ball that touches the line during a Star gets one `DespawnEnemy` and no `Die`;
without the Star, one `Die`. `cargo test`, clippy, `just replay` (no Star on level 1, so
the hashes do not move).

## Notes
- Done 2026-11-03. `handle_enemy_collision` collects each touched enemy once per frame, then
  triggers `DespawnEnemy` in a Star or writes `Die` without one. The Star's text is now
  "YOUR LINE KILLS FOR 6 S" (tip and wiki).
- Evidence: tests `in_a_star_the_line_kills_the_ball_that_touches_it` (one kill, no `Die`)
  and `a_ball_passes_through_the_line_and_touches_it` (one `Die`, no kill), on one helper;
  128 tests; clippy clean with and without `dev_tools`; `just replay` unchanged.
