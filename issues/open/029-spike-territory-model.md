# [029] Spike: robust territory model and exact claimed area

**Status:** open
**Milestone:** M3
**Depends on:** [013], [019]
**Created:** 2026-11-02
**Closed:** -

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
