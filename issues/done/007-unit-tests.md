# [007] Unit tests for the pure gameplay logic

**Status:** done
**Milestone:** M0
**Depends on:** -
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
There are no tests. The geometry (point in polygon, polygon completion around the walls,
choice of the side to claim, self-intersection of the drawn path), the score formatting and
the enemy count curve are pure functions that the upgrade and the M3 rework will touch.

## Approach
In-module `#[cfg(test)]` tests for the pure functions in `src/rix/polygon.rs`,
`src/rix/user_input.rs`, `src/rix/wall.rs`, `src/rix/setup.rs` (`Points` display) and
`src/rix/enemies/rng.rs`. Pin current behaviour, including known quirks (note them in the
test name), so the upgrade cannot change them silently.

## Verification
`cargo test` passes and covers each function named above.

## Notes
- Done. 28 in-module tests: `polygon.rs` (point in polygon, `get_possible_polygons` and
  `complete_polygon` for wall-to-wall, adjacent-wall and same-wall paths,
  `get_correct_polygon`, `find_closest_point_index`), `user_input.rs` (world coords, wall
  hit box, inner edge point, `get_edge_point`, segment intersection, path
  self-intersection, approximate equality), `wall.rs` (`next_wall`, `next_corner`, inner
  corners), `setup.rs` (`Points` display, `LevelInfo::with_seed`), `enemies/rng.rs`
  (enemy count ranges over 500 seeds, determinism).
- Quirks pinned (test names say `quirk`): points on the left and bottom edges of a
  polygon count as inside, on the right and top as outside; collinear overlapping
  segments do not count as an intersection; the enemy count drops from 9..=13 at level 25
  to 6..=10 at level 26; a same-wall path drawn right to left gives a counter-clockwise
  polygon that runs back over its own base (point tests still right, [029] Notes).
- Not tested: `extract_points_from_shape` (lyon, replaced in [013], which tests its own
  module); `polygon::area`, `wall::get_window_inner_corners` and
  `wall::world_to_window_coords`, which nothing calls ([035] Notes).
- Entities in tests come from `World::spawn_empty`, not `Entity::from_raw`, so the tests
  survive the entity API changes in [014]-[018].
