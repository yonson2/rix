# [024] Mobile performance pass

**Status:** open
**Milestone:** M2
**Depends on:** [022]
**Created:** 2026-11-02
**Closed:** -

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
