# [029] Spike: robust territory model and exact claimed area

**Status:** done
**Milestone:** M3
**Depends on:** [013], [019]
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
Claims are separate polygons. A new line that starts or ends on an older polygon is
spliced onto that polygon's outline by nearest-vertex search (`handle_build_polygon`,
`find_closest_point_index`), then closed around the wall corners. History shows this
area is fragile ("fix spawn bug by short-circuiting", "Fix colliders!", "improve margin
detection", "improve shape resolution for better accuracy"). Progress is a 100,000-sample
Monte Carlo estimate, not the real area.

## Approach
Time-boxed spike, then a decision for Pedro:
- A: keep one "free region" polygon and cut it with each line (polygon boolean ops, e.g.
  `i_overlay` or `geo`); claimed area = field area − free area, exact.
- B: a raster grid (classic Qix/Xonix): rasterise the line, flood-fill from each enemy,
  claim the unreached cells; exact cell count; render via a texture or merged meshes.
Compare on: robustness (fuzz random lines), cost per claim on a phone, how claims look
(smooth edges), effort.

## Verification
A short write-up in Notes with fuzz results (N random claims, zero invalid polygons for
the chosen option), timings, and Pedro's choice recorded in `decisions.md`.

## Notes
- Found in [007]: a same-wall path drawn right to left gives a counter-clockwise polygon
  that runs back along the wall over the path's own base. `is_point_in_polygon` still
  answers right, but the outline overlaps itself: a non-zero fill or the VHACD collider
  can include the bump, and a shoelace area counts it twice. Test
  `possible_polygons_quirk_same_wall_right_to_left_overlaps_but_point_test_holds`.
- Found in [002]: `calculate_area_progress` samples a box of (width - 70) x (height - 70)
  around the origin, but the field is offset: the top wall is 65, not 35. The box takes
  in a 30 px band of the top wall (y 247.5 to 277.5 on 625) that no claim can fill, so
  the full field reads about 94.6 %, and every progress value is about 5 % low.
- From [024] (2026-11-02): progress is now `claimed_fraction`, an exact scanline union of
  the outlines (one row per world unit, even-odd per outline), not a Monte Carlo estimate.
  The box quirk above is unchanged. The claim collider is a trimesh of the same lyon fill
  as the mesh, so collider and drawing now agree, also on the same-wall bump.
- Spike result (2026-11-02), write-up in `docs/research/territory_model.md`, code in
  `docs/research/territory_spike/`. Model A (one free-region polygon cut by each line,
  with an input rule: a line point within 1 unit of the free boundary ends the line and
  snaps to it): 0 invalid cuts in 160,000 fuzzed claims (8 seeds x 20,000), p50 3.4 µs,
  p99.9 25-36 µs per cut on desktop release; without the input rule about 3 % invalid.
  Model B (grid, flood fill): 0 invalid by construction, p50 0.35 ms (2-unit cells) to
  1.3 ms (1-unit cells), staircase edges, needs new render and collider paths.
  Recommendation: A. Waits for Pedro's choice.
- Pedro chose A (2026-11-02, D19). The build is [074].
