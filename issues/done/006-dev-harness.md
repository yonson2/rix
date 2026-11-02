# [006] Dev harness - fixed seed, scripted line, screenshot and exit

**Status:** done
**Milestone:** M0
**Depends on:** -
**Created:** 2026-11-02
**Closed:** 2026-11-02

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
- Done. `src/dev_tools.rs` (`DevToolsPlugin`) sits behind its own `dev_tools` feature, not
  `dev`: `dev_tools` turns on no Bevy feature, so `just shot` reuses the normal build
  artifacts (`dev` adds `bevy/dynamic_linking` and still includes `dev_tools`).
- `LevelInfo::with_seed(seed)` is new; `Default` uses it with a random seed.
- The gesture is time-based (line at 1.0 s, game shot at 3.0 s, menu shot at 3.2 s, give
  up with exit code 1 at 20 s). Line: y = 86 % of the window, x from 2 % to 98 %, 12 frames.
- macOS 27 blocker: on Bevy 0.15 (winit 0.30.9, objc2 0.5.2) every debug build panicked
  at start: `invalid message send to -[... NSScreen ... countByEnumeratingWithState:objects:count:]:
  expected return to have type code 'q', but found 'Q'`. objc2 checks message encodings
  only with `debug_assertions`, so `Cargo.toml` turns them off for the `objc2` package in
  the dev profile. Remove the override when the winit/objc2 that Bevy uses no longer
  panics (check at each upgrade step, [014]-[018]).
- Bevy 0.15 baseline, seed 1: one claimed strip of 9 %, `Points now at: 9`, one square
  enemy; two runs gave the same points. Baseline PNGs stay outside git
  (`target/shots/0.15-game.png`, `target/shots/0.15-menu.png`).
- Clippy with `--features dev_tools`: no new warnings (17 pre-existing ones, [035]).
