# [019] Resolution-independent playfield

**Status:** done
**Milestone:** M2
**Depends on:** [018], [010]
**Created:** 2026-11-02
**Closed:** 2026-11-02

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
- [010] answers (2026-11-02): Q5, one fixed playfield on every device, so a seed gives the
  same game everywhere. Q4, portrait and landscape: the fixed field letterboxes both ways.
  "The same game" also needs a fixed system order ([026]) and physics that does not follow
  the frame rate. Check avian's `enhanced-determinism` feature too, for float results that
  match across CPUs (iOS, Android, web).
- D17 (2026-11-02): Pedro picked a tall 9:16 field (options were square 625x625, 9:16, 3:4). It fills portrait phones; landscape shows a strip in the middle. Retune speeds and spawn counts.
- 2026-11-02, closed. `wall::FIELD` = 450 x 800 world units, walls included (D17: tall
  9:16; area 360,000, close to the old 625 x 625 = 390,625, so spawn density stays
  similar). Walls, enemy and bomb spawn bounds, item despawn line, the area sampler and
  the claim corners read `FIELD`; no gameplay code reads the window. `resize_walls` is
  gone (a resize only changes the view).
- Camera: `Projection::Orthographic` with `ScalingMode::AutoMin { 450, 800 }`: the whole
  field fits, the rest is letterbox (the clear colour, lifted a bit by the CRT ambient).
- Input: `handle_input_events` maps the cursor and touches with
  `Camera::viewport_to_world_2d` and clamps to the field (`to_field`), so a finger in the
  letterbox lands on the nearest wall. `convert_to_world_coords` and its test are gone;
  `is_point_in_wall` and `get_edge_point` take the wall length from `FIELD`.
- HUD (`ui.rs`): `Text2d` in the world at the old offsets from the field edge (z 10), so
  it scales and letterboxes with the field. The menu was already `Text2d` and a `Shape`;
  the game over screen stays UI in window pixels ([038] note).
- Desktop window: 450 x 800, resizable now. Dev harness: `RIX_SHOT_SIZE=<w>x<h>`; the
  scripted line is in world units (`world_to_viewport`), so it draws the same line at any
  size.
- Kept on purpose: the sampler's centered box quirk ([029]); speeds and spawn counts
  ([047] retune, D17); `get_window_inner_corners` and two other dead helpers ([035]).
- Evidence: `RIX_SHOT_SIZE` 450x800, 390x844, 1920x1080 and 625x625 with seed 1: the same
  field, letterboxed, the same claim (7/75 %, score 7) and the same square
  (`target/shots/019-*.png`, local). Field crops scaled to 450 x 800 and blurred differ
  by 0.9-1.3 % RMSE (scanlines and scaling). Menu shots at 450x800 and 390x844
  letterbox too. Android emulator (1080 x 2400): the field letterboxes top and bottom; a
  swipe from the bottom band to the top band clamps onto both walls and claims (22 %);
  rotated to landscape mid-run, the same process keeps the claim, score and field, and a
  line drawn there from band to band claims (28 %). Tests 31 pass (one window-mapping
  test went with its function); clippy clean with and without `dev_tools`.
- Not covered here: frame-rate and cross-CPU determinism ([072], filed now).
