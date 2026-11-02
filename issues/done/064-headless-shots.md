# [064] Headless shots: no window, no winit

**Status:** done
**Milestone:** M0
**Depends on:** [018]
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
A `RIX_SHOT` run still opens a winit window. On macOS the app takes the focus at launch,
so [062] gives it back and hides the window with AppKit calls (and two macOS-only deps,
`objc2-app-kit` and `objc2-foundation`). The window still shows for about 0.5 s, and a
tiling window manager (AeroSpace) can place it. Since [017] the camera already renders
into an image, so the window has no job in a shot run.

## Approach
Pedro's idea (2026-11-02), after Bevy's `examples/app/headless_renderer.rs` (0.19+):
- In a shot run, build `DefaultPlugins` with `.disable::<WinitPlugin>()` and add
  `ScheduleRunnerPlugin::run_loop(1/60 s)`. `BasePlugin` needs to know it is a shot run.
- Keep the camera-to-image target and `Screenshot::image` from [017]. The example says
  `Screenshot` is the simple path for one picture; its `ImageCopy` render node is for
  every frame.
- Game code reads the `Window` (`Single<&Window>` in wall.rs, level.rs, camera.rs, the
  touch mapping). Keep `primary_window: Some(..)` so a `Window` + `PrimaryWindow` entity
  exists without winit (scale factor 1, so 625 x 625 logical = physical). Check first
  that the renderer skips a window without a raw handle.
- Remove the macOS focus give-back, `FocusOwner` and the two objc2 deps.
- Check `ExitCondition` (no window close event) and that `AppExit` still ends the run.

## Verification
`just shot` and `just shot-menu` write PNGs that match `target/shots/018-*.png`, seeds 4
and 10 behave as before, no window appears and the focus does not move (macOS), and
`cargo tree -i objc2-app-kit -e normal` shows it only under winit.

## Notes
- Reference: https://github.com/bevyengine/bevy/blob/main/examples/app/headless_renderer.rs
  (0.20.1 copy in the cargo registry). https://github.com/kelo221/bevyout/issues/240 is a
  feature request with the same motive (black `Screenshot::primary_window()` on macOS when
  occluded); it has no code.
- Done: `BasePlugin` disables `WinitPlugin` when `dev_tools::is_shot_run()` (`RIX_SHOT`
  set; only with the `dev_tools` feature). `DevToolsPlugin` adds
  `ScheduleRunnerPlugin::run_loop(1/60 s)`, the same step as the manual time. The primary
  `Window` stays (625 x 625, scale 1, no OS window): `bevy_render` extracts only windows
  with a `RawHandleWrapper`, so it skips it, and the camera keeps the image target from
  [017]. Removed: the macOS focus give-back, `FocusOwner`, the `objc2-app-kit` and
  `objc2-foundation` deps.
- Exit: `ExitCondition` stays `OnPrimaryClosed`; the primary window never closes, and the
  `AppExit` from the screenshot observer ends the runner loop (exit code 0).
- Checks: `just shot` and `just shot-menu` are pixel-identical to the 0.20 winit shots
  (`target/shots/020-*.png`, AE 0); seed 4 identical; seed 10 only the arrow moved
  ([026]). During four shot runs, `lsappinfo` (sampled every 0.25 s) never listed a `rix`
  app and the front app stayed the same. `cargo tree -i objc2-app-kit@0.2.2 -e normal`:
  only under winit and accesskit_macos. `cargo build` (no features) has no warning;
  `cargo test` (32) passes; clippy: the same 9 old warnings.
- The four shaders the shots do not use were checked in [018] with a headless probe
  built the same way.
- Seed 10 logged an avian "no mass or inertia" warning: filed as [065].
