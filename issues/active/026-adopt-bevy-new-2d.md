# [026] Adopt bevy_new_2d structure and idioms

**Status:** active
**Milestone:** M3
**Depends on:** [018]
**Created:** 2026-11-02
**Closed:** -

## Problem
The structure grew by accretion: everything gameplay sits in `src/rix/` with one plugin per
file wired by hand in `RixPlugin`; cleanup on state exit uses marker components plus a
generic `despawn_screen::<T>` system per state; systems have no ordering sets, so input,
physics reactions and UI updates run in whatever order Bevy picks; a `Setup` state exists
only to register one-shot systems. Pedro asked to study TheBevyFlock/bevy_new_2d and adopt
the patterns that fit (D7).

## Approach
Adopt the patterns the study ranks first (findings in Notes). Expected shape: `screens/`
(loading, title, gameplay, game over) with state-scoped entities instead of
`despawn_screen`, `game/` (line, territory, enemies, items, level, score), `theme/` (palette,
UI widgets), `audio`, `asset_tracking`, `dev_tools` behind the `dev` feature,
`pub(super) fn plugin(app)` per module, an `AppSystems` set chain (tick timers → record
input → update), and `Reflect` on components for the inspector. Do it as a sequence of
behaviour-preserving moves, each verified by `just shot`.

## Verification
`cargo test` and `just shot` pass after each move; `ARCHITECTURE.md` describes the new
layout; no `despawn_screen` left.

## Notes
- 2026-11-02 study of bevy_new_2d (main @ bb9fa3e, targets Bevy 0.19, newest tag
  `bevy-v0.18`): full report in `docs/research/bevy_new_2d.md`. Ranked for Rix:
  1. `DespawnOnExit` + a gameplay sub-state (`Screen {Loading, Title, Gameplay, GameOver}`,
     `Phase {Playing, LevelTransition}` sourced from `Screen::Gameplay`): walls, shapes, HUD
     scoped to Gameplay; enemies, items, temp lines, explosions scoped to Playing. Removes
     `despawn_screen`, the marker components and `spawn_walls`'s "already spawned" guard;
     explosions stop leaking into the next screen. M.
  2. `LoadResource` asset collections + a Loading screen ([027]); `commands.run_system_cached`
     makes the `Setup` state unnecessary ([032]); warm the custom pipelines during load so
     the first kill does not hitch on a phone. M.
  3. One `PausableSystems` set gated by `IsPaused::Running` instead of 12 `run_if` calls; keep
     Rix's `IsPaused` sub-state (it resets on exit). S.
  4. `AppSystems {TickTimers, RecordInput, Update}` chained: readers of `InputPosition`,
     `DrawLine`, `BuildPolygon`, `ClearLines`, `WallEvent` have no order today (random 1-frame
     lag). S.
  5. Split `LevelInfo` ([028]). 6. Function plugins (`pub(super) fn plugin`) and private
     modules: `pub mod` everywhere hides dead code ([035]). 7. Bundle functions
     (`fn ball(..) -> impl Bundle`) and `theme/` widgets; `colors.rs` → `theme/palette.rs`.
  8. `audio.rs` markers + bundle fns ([041]). 9. `dev_tools.rs` behind `dev`, plus tracing's
     `release_max_level_warn` to drop hot-path `info!` from release and mobile builds.
  10. `Reflect` + `Name` (low priority).
- Skip: binary-only crate (Rix needs `lib.rs` for mobile), splash screen, keyboard-only
  flow (mobile needs a pause button and lifecycle pause), `default = ["dev_native"]` (a plain
  `cargo build --release` from Xcode or Gradle would link Bevy dynamically).
- Verify on 0.20: Bevy UI probably renders after the CRT node, so UI gets no bloom or CRT;
  that may be why the menu uses world-space `Text2d`.
- Found in [062]: `Update` has dozens of unordered system pairs that touch the same data
  (most on `LevelInfo`, which holds the RNG, and on events such as `InputPosition`, `Die`,
  `UserPath`). The executor picks their order at run time, so a seeded run is not
  reproducible when an enemy reacts to input (arrow). bevy_new_2d's `AppSystems` chain
  (`TickTimers`, `RecordInput`, `Update`) plus explicit order for RNG users fixes this.
  List them: `ScheduleBuildSettings { ambiguity_detection: LogLevel::Warn, .. }` on
  `Update`. Done when arrow seeds (7, 10) give identical `just shot` runs.
- Move 1 (2026-11-02): `AppState` became `Screen {Setup, Title, Gameplay, GameOver}` with
  the sub-state `Phase {Playing, LevelTransition}`; `IsPaused` and `TransitionState` hang
  off `Phase`. Every cleanup is `DespawnOnExit`: `RixComponent`, `despawn_screen`,
  `GameOverComponent` and `spawn_walls`'s guard are gone (`MenuComponent` stays: a query
  filter). Check: a new `RIX_SHOT_SCENE=flow` run (`just shot-flow`: claim, level clear,
  death, retry) logs an entity census at each change; old and new builds give the same
  census (one pair of runs identical; the other differs by one line-segment frame and by
  the retry's random seed, which also varies old vs old). `just shot` and `just
  shot-menu` are byte-identical to the old build; 38 tests pass. Bevy 0.20 stores
  resources as entities (`IsResource`), so the census leaves them out.
- Move 2 (2026-11-02): `PausableSystems` (gated by `IsPaused::Running`) replaces 12
  `run_if(in_state(IsPaused::Running))`; `AppSystems {TickTimers, RecordInput, Update}`
  chained in `Update`, input in `RecordInput`, its readers in `Update`, `despawn_after` in
  `TickTimers`. Check: `just shot` and `just shot-menu` byte-identical to the old build;
  the flow census is now the same in 3 of 3 runs and loses the one-frame gap between
  clearing the line and building the shape (input and claim in one frame). Arrow seeds:
  the old build gave 6 different pictures in 6 runs for seed 7; the new build gives one
  picture in 10 of 10 runs for seeds 7 and 10 and 6 of 6 for seed 3. This meets the
  [062] note. `Update` still has 101 ambiguous system pairs (Bevy's
  `ambiguity_detection`); ordering them is [072]'s work.
- Move 3 (2026-11-02): layout and function plugins. `src/rix/` -> `src/game/`;
  `src/menu/` -> `src/screens/title.rs`; `game_over.rs` -> `src/screens/`; `Screen` lives
  in `src/screens/mod.rs`; `colors.rs` -> `src/theme/palette.rs`. All 23 plugin structs are
  `pub(super) fn plugin(app: &mut App)`, added by the parent module. Check: shots
  byte-identical, census the same as move 2 (only the retry's random seed differs), seeds
  7 and 10 give the move 2 pictures, 38 tests, clippy clean.
