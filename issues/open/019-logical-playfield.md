# [019] Resolution-independent playfield

**Status:** open
**Milestone:** M2
**Depends on:** [018], [010]
**Created:** 2026-11-02
**Closed:** -

## Problem
The playfield is the window: walls, spawn areas, the area sampler and the HUD all derive
from `window.width()`/`height()` in pixels, and input skips the camera
(`convert_to_world_coords` in `user_input.rs` maps window pixels straight to world units).
So:
- the same seed gives a different game on every screen size, which defeats sharing seeds
  (commit f554584);
- speeds (`BALL_SPEED` px/s) and sizes feel different per device (`feat/ios` had to retune
  `WALL_THICKNESS` 35 → 20 and the wall hit margin 0.5 → 1.1);
- a resize mid-game (desktop, or rotation) moves the walls but not the claimed polygons.

## Approach
A fixed logical playfield (e.g. 600x600 world units plus a HUD band; shape per [010] Q5),
a camera `Projection` with a scaling mode that fits it into any window (letterbox), and
all input mapped with `Camera::viewport_to_world_2d`. Walls, spawn bounds and area come from
the logical size, never the window. UI scales with the window (`UiScale` or relative
units).

## Verification
`just shot` at 625x625, 390x844 (phone portrait) and 1920x1080 shows the same playfield
(letterboxed) with the same seed; a drag claims the same area on each.

## Notes
- Gate [010] Q5 decides letterbox versus fill.
