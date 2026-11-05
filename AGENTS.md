# Rix - Working agreements

> **Live document** for anyone (human or agent) working on this repo. Update it when a
> practice changes or a new hard rule appears. There is no `CLAUDE.md`: Claude Code reads
> this file (D5).

## What Rix is

Rix is a Qix-inspired arcade game in Rust and [Bevy](https://bevy.org), made for phones
first. The player draws a line from wall to wall with one finger. The line closes a
polygon with the walls: the side without enemies becomes claimed area (if both sides
have enemies, the first side is claimed and its enemies are eaten). An enemy that touches
the line while it is drawn kills the player. Claim 75 % of the field to clear a level.
Enemies (ball, teleporting square, arrow that chases the finger) and items (bomb) grow in
number with the level; points double per level. The look is neon on dark with bloom and a
CRT post-process.

**Goal (D1, D9):** release on iOS and Android first, then the web build on itch.io. No
Steam, no GOG.

## Start here

1. [issues/README.md](./issues/README.md) is the source of truth for WHAT to do: the issue
   tracker index, one file per issue in `issues/open|active|done/`. **At start:** read the
   index, then anything in `issues/active/`. "Continue" means: take the top of the open
   pile. **Gate issues need Pedro's answers before the issues that depend on them start,
   even in auto mode.** An issue is done only when its **Verification** passes.
   History sits beside the index, read on demand and never whole: `issues/done-log.md`
   (every closed issue, newest first; `grep` it) and `issues/decisions.md` (the D rows and the
   gate answers).
2. `ARCHITECTURE.md` ([002]) describes what IS: code map, data flow, invariants.
3. `docs/research/` holds studies that issues link to (for example
   `docs/research/bevy_new_2d.md`, the patterns Rix adopts in M3, D7). `docs/design/`
   holds feature specs (for example `docs/design/splitter.md`, [044]).

## Tracker and live-docs discipline

- **Issues:** new issues use `issues/TEMPLATE.md` and the "Next ID" in
  `issues/README.md`. Change status by moving the file `open/ -> active/ -> done/` and
  updating the index in the same commit. A close sets **Closed:**, records the result and
  its evidence in Notes, and adds the issue's line at the top of `issues/done-log.md`.
  Every issue keeps a runnable **Verification** line.
- **Discoveries:** the moment untracked work, a follow-up or a dead end appears, file it
  as an open issue (or append it to the Notes of the issue it belongs to). Never do
  untracked work; never leave a discovery only in chat. Don't file what an existing issue
  covers. Record dead ends.
- **Decisions:** an owner decision goes into `issues/decisions.md` with its date.
- **After closing an issue:** update `ARCHITECTURE.md` when structure changes (modules,
  plugins, states, events, dependencies) and this file when a practice or command changes.

## Hard rules

- **One Bevy minor at a time (D3):** each upgrade step builds, passes tests and clippy, and
  plays (`just shot`) before the next one starts.
- **No `bevy_prototype_lyon` (D2):** shapes are own meshes from `lyon_tessellation` ([013]).
- **Assets need a known licence:** every font, sound, song and image has a source and
  licence recorded before it ships ([011], [052]). Unknown licence means it does not ship.
- **No secrets in git:** signing keys, keystores, store and API credentials stay out of the
  repo (CI secrets or local files that `.gitignore` covers).
- **Reproducible runs:** gameplay randomness goes through the seeded `GameRng` resource
  (`src/game/setup.rs`), never the global `fastrand` functions, so `RIX_SEED` replays a run.
  A system that draws for several entities, or picks one, goes over them sorted with
  `setup::by_place`, never in query or event order: that order follows the archetypes,
  which a skin or an effect changes ([104]).

## Code standards

- Rust 2024, Bevy ECS idioms, in the bevy_new_2d layout ([026]): `src/screens/`,
  `src/game/`, `src/theme/`; one plugin per module as `pub(super) fn plugin(app: &mut
  App)`, added by the parent module; `States`/`SubStates` for flow (`Screen` in
  `src/screens/mod.rs`; `Phase`, `IsPaused`, `TransitionState` in `src/game/state.rs`),
  events for cross-system messages (`src/game/events.rs`), `DespawnOnExit(<state>)` on each entity for
  cleanup on state change, and system sets ([026]): gameplay systems go in
  `PausableSystems` and one of `AppSystems::{TickTimers, RecordInput, Update}` (input
  readers in `Update`, after `RecordInput`). A system that moves bodies, reacts to
  collisions, ticks a gameplay timer or draws from `GameRng` goes in `FixedUpdate`; only
  systems that just draw go in `Update` ([072]). `just replay` checks it.
- Assets load once, on the loading screen ([027]): put each handle in a `LoadResource`
  collection (`src/asset_tracking.rs`; `FontAssets` in `theme`, `ItemAssets` in
  `basic_bomb`) and give systems `Res<..Assets>`, never `asset_server.load` in a system.
- Keep pure logic (geometry, scoring, spawn counts) in plain functions with unit tests next
  to them (`#[cfg(test)] mod tests`). A test that pins odd current behaviour says `quirk`
  in its name.
- Minimum code that solves the issue; match the style around it; no speculative
  abstraction; every changed line traces to an issue. The code is rustfmt-clean ([066]):
  run `cargo fmt` before each commit.
- Clippy stays clean: `cargo clippy --all-targets --features dev_tools -- -D warnings`
  passes (CI checks it, [008]). The `[lints]` table in `Cargo.toml` allows
  `too_many_arguments` and `type_complexity` (Bevy systems and queries); add no
  per-item `#[allow]` for them ([035]). `cargo build` shows no dead-code warning: keep
  modules private in `lib.rs` unless something outside the crate needs them.

## Git

- Work on a branch, one commit per issue, message `[NNN] short summary` (D4).
- Never add `Co-Authored-By` trailers or any other AI attribution to commits.
- Stage explicit paths, never `git add -A` or `git add .`: Pedro may keep untracked notes and
  assets in the working tree that stay out of git.
- Version (D12): bump `version` in `Cargo.toml` (and `Cargo.lock`) in the merge to `main`:
  minor when a milestone completes, patch for other game changes; 1.0.0 is the mobile store
  release. Tag the bump commit `vX.Y.Z` (annotated, D14).
- After a merge into `main`, delete the merged work branch (D14).
- Never push and never delete any other branch without Pedro's go.

## Commands

- Run (dev, dynamic linking): `just dev` (= `cargo run --features dev`)
- Build: `cargo build` · Release: `cargo build --release`
- Disk: `target/` grows past 80 GB (incremental caches, old `rix` test binaries). To free
  most of it without rebuilding the dependencies: `rm -rf target/debug/incremental` (and
  each `target/<triple>/debug/incremental`), then `cargo clean -p rix`.
- Tests: `cargo test`
- Lint: `cargo clippy --all-targets --features dev_tools -- -D warnings` ·
  `cargo fmt --check`
- CI ([008]): `.github/workflows/ci.yml` runs fmt, clippy (`-D warnings`, with and without
  `dev_tools`), the tests and the replay check (on Linux x86_64 and macOS arm64, [075])
  on each push and PR to `main`, in the fast `ci` profile (`--profile ci`).
- Screenshot of a scripted game, then exit ([006]): `just shot [path]` (default
  `target/shots/game.png`; fixed seed, one line across the bottom of the field)
- Screenshot of the menu: `just shot-menu [path]`
- State flow ([026]): `just shot-flow` plays a claim, a level clear, a death and a retry,
  and prints an entity census (`RIX_FLOW`) each time it changes. Compare it before and
  after a change to states or entity lifetimes (the run after the retry has a random seed).
- Replay ([072], [075]): `just replay` (`tools/replay.sh`) runs seeds 1, 2, 5, 7 and 8
  with the scripted line at 30, 60 and 120 frames per second, without the renderer, and
  fails if a seed's state hash (`RIX_STATE`) differs between frame rates or from
  `tools/replay-hashes.txt`. Run it after a change to gameplay systems or their schedule.
  A change that moves level 1 on purpose updates that file in the same commit.
  Gameplay draws from `GameRng` use `u64`, not `usize` or `choice` (32-bit wasm draws
  those differently). Float functions (`sin`, `cos`, `atan2`, `exp`, `powf`, ...) go
  through `ops::` (`bevy::math::ops`, in the prelude), never the `f32` methods, which call
  the platform libm: `clippy.toml` forbids them ([075]).
- Android ([021]): `just android` (debug APK: cargo-ndk, then Gradle) · `just android-release`
  (the signed AAB for Play; the upload key `~/keys/rix-upload.jks` and its password come
  from `.env`, [054], D38) · `just android-run` (install and start). Gradle 9.4 runs on
  Android Studio's JDK 21 (the recipes set `JAVA_HOME`; Homebrew's JDK 27 is too new).
  A phone may be paired over wireless adb: set `ANDROID_SERIAL=emulator-5554` so adb
  never touches it. Start the emulator without a window:
  `~/Library/Android/sdk/emulator/emulator -avd Medium_Phone_API_36 -no-window -no-audio`.
  If `adb install` fails with `INSUFFICIENT_STORAGE`, run `adb uninstall top.cosas.rix` first.
  The emulator uses gesture navigation. A back gesture needs a slow swipe: `adb shell
  "input touchscreen motionevent DOWN 3 1500; input touchscreen motionevent MOVE 40 1500;
  ...; input touchscreen motionevent UP 300 1500"` (`input swipe` does not trigger it, [079]).
- iOS ([020]): `just ios-sim` (build, install and start on the simulator `IOS_SIM`, default
  iPhone 16; `simctl` boots it without a window) · `just ios-device` (signed with Pedro's
  team, needs `IOS_DEVICE`) · `just ios-archive` (Release archive in `target/ios/`) ·
  `just ios-upload` (archive, then upload to App Store Connect with the Apple account in
  Xcode; owner-run, [053]). Screenshot: `xcrun simctl io booted screenshot <file.png>`.
  Build through `just`: the recipes pass the version from `Cargo.toml` to Xcode.
  The simulator plays the game's sound through the Mac speakers ([041]): stop the app
  after a check (`xcrun simctl terminate booted top.cosas.rix`).
- Dev env vars (feature `dev_tools`): `RIX_SEED=<u64>` fixes the run seed;
  `RIX_LEVEL=<n>` (with `RIX_SEED`) starts the run at level n ([051]);
  `RIX_SHOT=<file.png>` and `RIX_SHOT_SCENE=menu|wiki|skins|records|settings|credits|flow|store|replay` are what the `just` recipes set
  (`store`: three wobbly lines, the shot while the third is drawn, [051];
  `RIX_SHOT_LINES=<n>` picks its line set, [103]);
  `RIX_SKINS=<line>,<wall>,<fill>` forces the skins (for example `chain,circuit,grid`, [078], [115]);
  `RIX_SHOT_SIZE=<w>x<h>` sets the image size (for example `RIX_SHOT_SIZE=390x844 just
  shot`: the field takes the image shape, [080]); `RIX_SHOT_FPS=<n>` sets the frames per second
  (default 60, [072]). A shot run uses a fixed time step per frame, and a seed replays
  the same picture (also with an arrow since [026]). It opens no window and takes no focus: no winit, and the
  camera draws into a 450 x 800 image by default ([064]). Running
  `target/debug/rix` directly needs `BEVY_ASSET_ROOT=<repo>`, or assets do not load.
  `RIX_SAVE=<file.json>` uses that save file ([048]); without it a shot run has no save
  file, no tutorial and no tips ([049], [070]; a fresh `RIX_SAVE` file shows them), and other runs use the app data directory (`~/Library/Application Support/rix` on
  macOS).
- Frame-time log ([024]): a `dev_tools` build warns `slow update: <ms> (frame <ms>)` when
  the main schedule (game logic and physics) takes over 16 ms. On Android, build it with
  the `just android` cargo-ndk line plus `--features rix/dev_tools`, then
  `adb logcat | grep 'slow update'`. On iOS, add `--features dev_tools` to the cargo line
  in `mobile/ios/build_rust_deps.sh` for that build; Bevy logs to os_log, so read it with
  `xcrun simctl spawn booted log show --last 2m --predicate 'process == "Rix"'` (or
  Console.app for a phone). The Android emulator renders in software: every frame there
  takes 100 ms or more, so read the update time, not the frame time.
- Shader examples: `cargo run --example glow_material` (also `cartoon_explosion`,
  `pixel_explosion`)
- Sounds ([042]): `tools/elevenlabs.py sfx PROMPT --out DIR/NAME --seconds S --takes 3`
  (load the keys first: `set -a && source .env && set +a`). Encode with `oggenc` (Homebrew's
  ffmpeg has no libvorbis). Every shipped sound has a row in `audio-src/sources.md`.
- Licences ([052]): `just licences` writes `assets/licenses/third-party.md` (the crate
  licences, shipped with the app) with `cargo-about` (`cargo install cargo-about --locked
  --features cli`). Run it after a dependency change. Asset credits live in `CREDITS.md`
  and the CREDITS screen.
- App icon candidates ([050]): `tools/image.py PROMPT --out DIR/NAME` (EvoLink image,
  default `doubao-seedream-5.0-pro`; writes NAME.png and the request in NAME.json).
- App icon ([050], D31): `tools/draw-icon.py` draws it into the iOS asset catalog and the
  Android `mipmap-*` folders. The bomb image ([098]): `tools/draw-bomb.py
  assets/items/bomb_basic.png`.
- Store art ([051], [101], [103], [108]): `just store-art` writes the App Store (iPhone only, D32) and
  Play screenshots (from the game, a seed, level and line set per shot in `tools/store-art.py`, each
  under a caption), the App Store header, the Play feature graphic and the Play icon into `target/store-art/`. After a change that moves the game, check the
  shots: a seed can then die or clear the level, and the script stops at it.
- Site ([102], D33): `site/` is plain HTML and CSS (no build); `just site-deploy` puts it
  on Vercel (log in once with `npx vercel login`). The privacy policy is `site/privacy.html`.
- Issue board ([004]): `npm ci` · `npm run dev` · `npm test` · `npm run check` ·
  `npm run build` (in `tracker/`; it reads `../issues`, a bad issue file fails the build)

## Verification culture

Turn tasks into verifiable goals (tests first where sensible); loop until the issue's
**Verification** passes. Changes that touch rendering, input or physics also need a
`just shot` (and `just shot-menu` for the menu) and a look at the PNG. Report failures
plainly with output; never claim done without running the check.

## Platform notes

- macOS 27: Bevy 0.15's winit uses objc2 0.5, whose debug message checks panic at start.
  `Cargo.toml` turns `debug-assertions` off for the `objc2` package in the dev profile
  ([006] Notes).
