# [020] iOS build on main

**Status:** open
**Milestone:** M2
**Depends on:** [018]
**Created:** 2026-11-02
**Closed:** -

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
