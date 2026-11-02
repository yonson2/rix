# [020] iOS build on main

**Status:** done
**Milestone:** M2
**Depends on:** [018]
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
iOS only builds on `feat/ios` (41 commits, many merges from main). That branch points
Cargo at a local Bevy fork (`bevy = { path = "../../git/bevy/" }` plus a `[patch]` for every
Bevy crate) to get `preferred_screen_edges_deferring_system_gestures`, and keeps the Xcode
project at the repo root (`rix.xcodeproj`, `ios-src/`, `Makefile`, `build_rust_deps.sh`,
icons under `assets/Assets.xcassets`).

## Approach
Port it to main on Bevy 0.20 from crates.io (D6: the screen-edge change is upstream;
confirm the field name and version). Move the iOS files under `mobile/ios/` (Xcode
project, Info.plist, asset catalog, the build script based on Bevy's `examples/mobile`),
with `just ios-sim` and `just ios-device` recipes. Keep the platform tweaks from the branch
behind `cfg(target_os = "ios")`: `#[bevy_main]`, borderless fullscreen,
`WinitSettings::mobile()`, deferred screen edges, gesture flags. Leave the wall-size tweaks
to [019]. Don't commit `xcuserdata/` or `build/`.

## Verification
`just ios-sim` builds, installs and launches Rix on an iOS simulator (iPhone 16), and a
simulator screenshot shows the game; no path dependency outside the repo in `Cargo.toml`.

## Notes
- The device run is in [025] (owner-run).
- Branch facts (2026-11-02): `feat/ios` Makefile targets device id 00008020-000E61663C05002E;
  Info.plist: portrait only, status bar hidden, `UIRequiresFullScreen`; bundle id
  `org.pp.rix`.
- [010] answers (2026-11-02): bundle id `top.cosas.rix` (not the branch's `org.pp.rix`);
  portrait and landscape in `Info.plist`; iOS floor: the lowest that Bevy 0.20 and wgpu 30
  support without extra work (a newer floor is fine). The `feat/ios` icon is not used
  ([050]). D11: delete `feat/ios` when this issue is done.
- 2026-11-02, closed. `mobile/ios/` follows Bevy 0.20.1 `examples/mobile`: `Rix.xcodeproj`
  (no sources; a "Rust" phase runs `build_rust_deps.sh`, which builds the root `rix` binary
  with `--target aarch64-apple-ios[-sim]` into the repo `target/`, then `lipo`s it into the
  app), the root `assets/` folder as a resource, `ios-src/Info.plist` (portrait and
  landscape, status bar hidden, `UILaunchScreen`, full screen). Bundle id `top.cosas.rix`,
  iOS 15.0 (the lowest Xcode 27 accepts), Pedro's team `B8Y7SGQP5V` from the branch,
  automatic signing. Version and build number come from `Cargo.toml` (0.3.1 -> 301, as on
  Android); the `just` recipes pass them as `MARKETING_VERSION` / `CURRENT_PROJECT_VERSION`.
- No Bevy fork (D6): `preferred_screen_edges_deferring_system_gestures` is upstream.
  `src/base.rs` on iOS: `BorderlessFullscreen` (a sized window drew a surface larger
  than the screen: menu cut off at the right, black band at the bottom), screen edges
  defer system gestures (`ScreenEdge::All`), status bar hidden. `#[bevy_main]` is not
  needed on iOS (it only adds `android_main`).
- Not taken from `feat/ios`: `WinitSettings::mobile()`, the reactive 60 Hz focused mode and
  `desired_maximum_frame_latency` 3 (frame pacing, [023]); gesture recognizers (unused);
  the wall-size tweaks ([019]); the old icon ([050]); the root `Makefile`.
- Evidence: `just ios-sim` cold 7m40s; the app starts on the iPhone 16 / iOS 18.5 simulator
  (booted by `simctl`, no window) and draws the menu full screen with CRT and bloom
  (`target/shots/020-ios-menu.png`, local); the process stays up. The device build compiles
  (`-destination generic/platform=iOS CODE_SIGNING_ALLOWED=NO`); the signed install is
  [025]. `cargo clippy --target aarch64-apple-ios-sim -- -D warnings` is clean.
- Not checked: touch play on iOS (`simctl` has no tap; Simulator.app would take the focus).
  It goes to [025] (device) and [023] (simulator lifecycle).
- `.gitignore`: `rix.xcodeproj` is now `/rix.xcodeproj` (git ignores case on macOS, so the
  old rule hid `mobile/ios/Rix.xcodeproj`). Pedro keeps a local untracked `rix.xcodeproj`
  at the root from the branch; it stays ignored and untouched.
- `feat/ios` deleted (D11).
