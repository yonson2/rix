# [074] Territory model A: one free-region polygon cut by each line

**Status:** done
**Milestone:** M3
**Depends on:** [029], [026]
**Created:** 2026-11-02
**Closed:** 2026-11-02

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
- Done (2026-11-02): `src/game/territory.rs` holds `FreeRegion` (reset to the box on
  `OnEnter(Phase::Playing)`) and the spike's `locate`, `walk`, `split`, `line_end` (the
  input rule) and `signed_area`. `handle_build_polygon` splits `FreeRegion`, claims a side
  with `get_correct_polygon` (now `[claimed, kept]` + eaten) and keeps the other. Progress
  is `FreeRegion::claimed_fraction` (exact), computed when `FreeRegion` changes. Removed:
  `PolygonData`, `ShapeMeta`, `get_possible_polygons`, `complete_polygon`,
  `find_closest_point_index`, the scanline `claimed_fraction` ([024]), `get_edge_point`,
  `is_point_in_wall`, `calculate_inner_edge_point`, `WallSide` (with `next_wall`,
  `next_corner`) and the tests that pinned them (13 tests out, 8 in).
- Input changes a player will feel (check on a phone, [025]):
  - A line now ends when it reaches the boundary (1 unit), not 17.5 units before a wall.
  - A press anywhere outside the free region (a wall or a claim, also a wall part that
    is already claimed) or within 17.5 units of its boundary starts a line. Until the
    finger leaves that zone, the start follows it along the boundary. Before, a press on
    a claimed wall part did nothing.
  - A press in a wall corner snaps to the box corner (before: to the wall's inner line,
    which could be outside the box).
- Guard: a cut with a side of no area (a first segment that leaves the region outside,
  possible only on a fast swipe at a concave corner) drops the line with a `warn!`.
- Evidence: `cargo test` 34 passed, among them `fuzzed_cuts_give_two_simple_pieces_that_add_up`
  (seed 29, 2,000 claims, 290 levels, 0 invalid; the spike gives 292 levels for 2,000
  claims, so the port behaves the same), `a_line_from_claimed_edge_to_claimed_edge_cuts_a_notch`,
  `a_line_from_a_wall_to_a_claimed_edge_cuts_a_corner`, `claimed_fraction_is_exact` (0 for
  the box, 0.5 for a half, 1 for an empty region) and
  `a_drag_from_a_claim_to_a_claim_sends_one_line_between_the_claimed_edges` (the input
  system end to end). Clippy clean with and without `dev_tools`, fmt clean.
- Shots: game seed 1 is byte-identical to [027] (`109eb09e`, the same 7 % strip); menu
  identical (`e520ba21`). Seeds 7 (`197a00c8`) and 10 (`78d871fa`) changed: the line ends
  one frame later, so the arrow follows the finger one frame longer; same claim, each
  replays 3 of 3. `just shot-flow`: one more line frame (10 segments) before the claim,
  then the same census; the seeded part is identical in 3 runs.
- Progress now reads the real fraction: about 4 % more than the old box (380 x 730) gave,
  so 75 % comes a little sooner. Noted in [047].
