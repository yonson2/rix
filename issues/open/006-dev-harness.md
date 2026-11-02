# [006] Dev harness - fixed seed, scripted line, screenshot and exit

**Status:** open
**Milestone:** M0
**Depends on:** -
**Created:** 2026-11-02
**Closed:** -

## Problem
Nothing proves the game still works after a change except playing it by hand. The engine
upgrade ([013]-[018]) changes rendering, physics and input at every step, and an agent
cannot play. Runs are also random (a new seed every run), so two runs never match.

## Approach
A `dev`-feature-only module (release builds never contain it), driven by env vars:
- `RIX_SEED=<u64>` fixes the run seed.
- `RIX_SHOT=<path.png>` skips the menu, plays a scripted touch gesture (press in the left
  wall, drag across the field near the bottom, release in the right wall: one claimed
  strip), waits, saves a screenshot of the primary window to the path and exits.
- `RIX_SHOT_SCENE=menu` screenshots the menu instead.
The gesture uses `TouchInput` messages, so it also exercises the mobile input path.
A `just shot` recipe wraps it.

## Verification
`just shot` exits on its own within ~10 s and writes a PNG that shows the walls, the HUD
and one claimed area; `just shot-menu` writes the menu.

## Notes
