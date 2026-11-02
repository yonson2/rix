# [024] Mobile performance pass

**Status:** done
**Milestone:** M2
**Depends on:** [022]
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
Known hot spots that a phone will feel:
- `calculate_area_progress` (`level.rs`) runs 100,000 random point-in-polygon samples over
  every claimed polygon in one frame each time a shape is added.
- `handle_draw_polygon` (`polygon.rs`) runs a VHACD convex decomposition (resolution 256)
  per claimed polygon.
- `handle_input` (`user_input.rs`) flattens every claimed shape several times per frame
  while drawing.
- HDR + bloom + SMAA + the CRT pass every frame; `feat/ios` had to "mess with sizes and
  framerate to improve performance".

## Approach
Measure first (Tracy or `bevy/trace` on desktop, frame times on device), then fix the
largest: exact area by the shoelace formula on the claimed region (with [029]), cached
polygon vertices (with [013]), cheaper colliders, a quality setting for post-processing.

## Verification
On the Android emulator and the iOS simulator, a claim causes no frame over 33 ms in a
frame-time log; numbers before and after are in Notes. Device numbers in [025].

## Notes
- From [018] (2026-11-02, read from the Bevy 0.20 source, not measured): `Sprite` now
  draws through `Mesh2d` + a sprite material cached per unique color and size.
  `ColorTransition` (`src/utils.rs`) changes `Sprite.color` every frame on the fading
  line segments, so each fading segment may get a new material each frame. Profile it on
  a phone; if it costs, draw the segments as `Mesh2d` + `ColorMaterial` or with
  `src/shape.rs`.
- Result (2026-11-02): measured first, then fixed the two claim costs.
  - Desktop (`just shot`, a 14-vertex claim): VHACD 16.3 ms dev, 4.4 ms release; the
    Monte Carlo 1.5 ms with one polygon. Late-level bench (8 nested claims of 150 points,
    test profile): Monte Carlo 67.3 ms, scanline 1.9 ms, same result (0.767).
  - Fix: `territory_collider` (`polygon.rs`) is a trimesh of the lyon fill triangles, so
    the collider is the drawn shape (0.5 ms dev). `claimed_fraction` (`level.rs`) is the
    union of the outlines on one scanline per world unit, with no RNG (26 us, one polygon).
  - Android emulator (dev build with `rix/dev_tools`, 6 nested swipes): every frame takes
    115 to 160 ms (llvmpipe software GPU), so no code meets 33 ms per frame there; the
    main-schedule update time is the measure. Before: claim updates 55 to 119 ms
    (ordinary median 26 ms). After: claim updates 25 to 45 ms, the ordinary level
    (median 28 ms); trimesh 22 to 84 us, `claimed_fraction` 34 to 85 us.
  - iOS simulator (iPhone 16, dev build, scripted lines of 60 points): before, each claim
    gave an update of 16 to 23 ms; after, no slow update at a claim (trimesh 30 to 36 us
    for 54 vertices, `claimed_fraction` 85 to 715 us for 1 to 4 polygons), and no update
    over 33 ms after start-up.
  - Trace (`bevy/trace_chrome`, desktop shot): outside claims no game system is a hot
    spot. Update averages 0.5 ms per frame, so the outline clones in `handle_input` and
    the `ColorTransition` sprite colors (Note above) do not show. The long Main frames of a
    shot run are start-up asset work and contention with the pipelined render thread.
  - Not measurable here: the GPU cost of HDR, bloom, SMAA and the CRT pass (software GPU
    on the emulator, the Mac GPU on the simulator). Moved to [025] (device frame times).
  - New: the `slow update` frame-time log in `dev_tools` (AGENTS.md "Frame-time log").
  - Tests: `territory_collider_is_solid_in_both_windings`,
    `a_ball_bounces_off_a_territory_in_both_windings`, three `claimed_fraction_*`.
