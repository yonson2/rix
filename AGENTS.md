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
   (every closed issue, newest first; `grep` it) and `issues/decisions.md` (D1-D8 and the
   gate answers).
2. `ARCHITECTURE.md` ([002]) describes what IS: code map, data flow, invariants.
3. `docs/research/` holds studies that issues link to (for example
   `docs/research/bevy_new_2d.md`, the patterns Rix adopts in M3, D7).

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
- **Reproducible runs:** gameplay randomness goes through the seeded `LevelInfo.rng`
  (`src/rix/setup.rs`), never the global `fastrand` functions, so `RIX_SEED` replays a run.

## Code standards

- Rust 2024, Bevy ECS idioms: one plugin per feature, `States`/`SubStates` for flow
  (`AppState`, `IsPaused`, `TransitionState` in `src/rix/state.rs`), events for
  cross-system messages (`src/rix/events.rs`), `run_if(in_state(..))` on gameplay systems,
  marker components for cleanup on state change.
- Keep pure logic (geometry, scoring, spawn counts) in plain functions with unit tests next
  to them (`#[cfg(test)] mod tests`). A test that pins odd current behaviour says `quirk`
  in its name.
- Minimum code that solves the issue; match the style around it; no speculative
  abstraction; every changed line traces to an issue. The code is rustfmt-clean ([066]):
  run `cargo fmt` before each commit.
- Clippy stays clean: `cargo clippy --all-targets --features dev_tools -- -D warnings`
  passes (CI checks it, [008]).

## Git

- Work on a branch, one commit per issue, message `[NNN] short summary` (D4).
- Never add `Co-Authored-By` trailers or any other AI attribution to commits.
- Stage explicit paths, never `git add -A` or `git add .`: Pedro keeps untracked notes and
  assets in the working tree that stay out of git (D8).
- Version (D12): bump `version` in `Cargo.toml` (and `Cargo.lock`) in the merge to `main`:
  minor when a milestone completes, patch for other game changes; 1.0.0 is the mobile store
  release.
- Never push and never delete a branch without Pedro's go.

## Commands

- Run (dev, dynamic linking): `just dev` (= `cargo run --features dev`)
- Build: `cargo build` · Release: `cargo build --release`
- Tests: `cargo test`
- Lint: `cargo clippy --all-targets --features dev_tools -- -D warnings` ·
  `cargo fmt --check`
- CI ([008]): `.github/workflows/ci.yml` runs fmt, clippy (`-D warnings`, with and without
  `dev_tools`) and the tests on each push and PR to `main`, in the fast `ci` profile
  (`--profile ci`).
- Screenshot of a scripted game, then exit ([006]): `just shot [path]` (default
  `target/shots/game.png`; fixed seed, one line across the bottom of the field)
- Screenshot of the menu: `just shot-menu [path]`
- Dev env vars (feature `dev_tools`): `RIX_SEED=<u64>` fixes the run seed;
  `RIX_SHOT=<file.png>` and `RIX_SHOT_SCENE=menu` are what the `just` recipes set. A shot
  run uses a fixed 1/60 s per frame; seed 1 replays the same picture, seeds with an arrow
  still vary until [026]. It opens no window and takes no focus: no winit, and the
  camera draws into a 625 x 625 image ([064]). Running
  `target/debug/rix` directly needs `BEVY_ASSET_ROOT=<repo>`, or assets do not load.
- Shader examples: `cargo run --example glow_material` (also `cartoon_explosion`,
  `pixel_explosion`, `flame_shader`)
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
