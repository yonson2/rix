# [074] Territory model A: one free-region polygon cut by each line

**Status:** open
**Milestone:** M3
**Depends on:** [029], [026]
**Created:** 2026-11-02
**Closed:** -

## Problem
Claims are separate polygons. A line that starts or ends on an older claim is spliced
onto that outline by nearest-vertex search (`handle_build_polygon`, `ShapeMeta`,
`get_possible_polygons`, `complete_polygon`, `get_correct_polygon`,
`find_closest_point_index`), which broke many times. Progress measures a box that is off
by the top wall ([029] Notes), so it reads about 5 % low. Pedro chose model A (D19).

## Approach
Port model A from `docs/research/territory_spike/` (write-up:
`docs/research/territory_model.md`):
- A `FreeRegion` resource: one simple counter-clockwise polygon, the playable box at the
  start of each level.
- Input rule: a line starts on the free boundary (a wall or a claimed edge) and ends when
  a point comes within 1 unit of the boundary, checked every 0.5 units along each
  segment; the line keeps that point and snaps to the nearest boundary point (`locate`).
- Cut with `locate`, `walk` and `split`. The side without enemies becomes the claimed
  piece; if both sides have enemies, keep today's rule (the first side is claimed and its
  enemies are eaten). Items inside the piece activate. The other side is the new free
  region.
- The claimed piece spawns as today: `DrawnShape`, `Territory`, lyon fill, trimesh
  collider ([024]).
- Progress = 1 - free area / box area (shoelace), exact. The 75 % target may need a
  retune ([047]).
- Remove the splice code and the tests that pin its quirks; port the fuzz into a unit
  test.

## Verification
`cargo test` runs a fuzz test (seeded, at least 2,000 claims, 0 invalid polygons) and unit
tests for a line that starts and ends on claimed edges; `just shot` and `just shot-flow`
show the claim; progress reads 100 % for a fully claimed box.

## Notes
- Spike numbers ([029]): 0 invalid in 160,000 fuzzed claims, p50 3.4 µs, p99.9 25-36 µs
  per cut (desktop release); without the input rule about 3 % invalid.
