# [106] Splitter Soldiers (and every polygon enemy) pass through claimed areas

**Status:** done
**Milestone:** M5
**Depends on:** [044], [030]
**Created:** 2026-11-04
**Closed:** 2026-11-04

## Problem
Pedro, on the 1.0.0 TestFlight build: "the splitter splits can navigate through the already
cleared area". A claimed area is a static trimesh of lyon's fill triangles
(`polygon::territory_collider`). A test with a Soldier next to a jagged claimed area
(`soldiers_stay_out_of_a_claimed_area`, 5 seeds, 30 s, 8 Soldiers) fails at tick 6: the
Soldier flies into the claim with its velocity unchanged, so no contact happens at all.
Two causes:
- Lyon's triangles turn clockwise in our y-up space (6 of 6 in the test claim). A circle's
  contact with a triangle does not care, so balls bounced; a polygon body (the Queen's
  octagon, a Soldier, a Square's box) gets no contact with a clockwise triangle and passes.
- The Soldier's `Collider::ellipse` gets no contact with a trimesh even with
  counter-clockwise triangles (avian2d's ellipse is a custom shape).

## Approach
`territory_collider` turns each triangle counter-clockwise (`[a, c, b]`). The Soldier's
collider is the convex hull of its look's 20-point outline. Dead end: checking the spawn
spot with the body's bounding circle (the body spawns at a random angle) did not change
the failure, so it is not in.

## Verification
`cargo test soldiers_stay_out_of_a_claimed_area a_ball_and_a_box_bounce_off_a_territory`
passes (the box case fails without the winding fix); `just replay` passes.

## Notes
- 2026-11-04: Verification passes. `cargo test` 134 passed; clippy clean with and without
  `dev_tools`; `just replay` hashes unchanged; `just shot` md5 unchanged
  (e170997305a806320c6f9f967cf3cf5b). Found by Pedro on the 1.0.0 TestFlight build; ships in 1.0.1.
