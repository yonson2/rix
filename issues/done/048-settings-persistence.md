# [048] Settings, high score and stats persistence

**Status:** done
**Milestone:** M4
**Depends on:** [028], [041]
**Created:** 2026-11-02
**Closed:** 2026-11-03

## Problem
Nothing persists: no best score, no settings. todo.md: "STATS resource (keep track of
enemies killed, and think more...)".

## Approach
A small save file in the platform's app data directory (iOS Application Support, Android
internal storage, desktop config dir): best score, lifetime stats (claims, kills, levels,
time), settings (music and SFX volume, CRT intensity, reduced flashing, haptics). A
settings screen reachable from the menu ([036]) and the pause menu ([034]).

## Verification
Set a best score and change a setting, restart the app: both are kept (desktop and iOS
simulator).

## Notes
- todo.md: "STATS resource (keep track of enemies killed, and think more...)".
- From [036] (2026-11-03): the title has PLAY (and QUIT on desktop) as
  `theme::widget::button`s. This issue adds a SETTINGS button and the best score to the
  title (`src/screens/title.rs`; buttons at y -60 and -170, the logo at 170).
- From [038] (2026-11-03): the game over screen (`src/screens/game_over.rs`, world
  `Text2d`) gets the best score and a "NEW BEST" mark here.
- From [040] (2026-11-03): the screen shake (`fx::Trauma`) and the haptics ([081]) want an off switch here; some players dislike shake.

## Result (2026-11-03)
- `src/save.rs`: `Save { best, stats, settings }` as JSON (`serde`, `serde_json`; both
  were in `Cargo.lock` already). The file is `save.json` in:
  - macOS and iOS: `HOME/Library/Application Support/rix`.
  - Android: the internal data path.
  - Windows: `APPDATA`. Other Unix: `XDG_DATA_HOME`.
  - Web: none yet (note added to [058]).
- Load and write:
  - It loads in the plugin. No file or a bad file gives the defaults.
  - It is written through a temporary file whenever it changes: at the start, at a game
    over and on a settings tap.
  - A shot run has no save unless `RIX_SAVE` names one, so shots and replays stay the
    same. The shot mute moved from `dev_tools` into `apply_settings`.
- Stats: runs, claims, kills, levels cleared and seconds of play, added by `record_run`
  at the game over.
- Best score: BEST on the title; "NEW BEST!" or BEST on the game over screen.
- Settings (screen `Screen::Settings`, from the title's SETTINGS button):
  - MUSIC, SOUND and CRT in 20 % steps (from 0 back to 100).
  - SHAKE, HAPTICS and FLASH (the white flash of a new claim) on and off.
  - Volumes and the CRT `intensity` apply at once. `fx` and `haptics` read their switch.
- Split out: settings from the pause menu go to [073] (filed first as [085], a duplicate),
  because the pause overlay has no buttons (a tap anywhere resumes).
- Evidence:
  - Tests: `a_save_survives_a_round_trip_and_a_partial_file_fills_in_defaults`,
    `a_file_is_written_and_read_back_and_a_bad_one_gives_defaults`,
    `a_run_adds_its_stats_and_only_a_higher_score_is_a_new_best`,
    `a_percent_setting_steps_down_and_wraps_to_full`,
    `each_setting_steps_and_shows_its_value`. 86 tests pass.
  - Clippy is clean with and without `dev_tools`, for the iOS simulator target and for
    Android (cargo ndk). fmt is clean.
  - `just replay` and the `just shot` md5 are unchanged.
  - Desktop, with `RIX_SAVE` set to a scratch file:
    1. The flow scene died with 7 points, and the game over wrote best 7, runs 1, claims 1,
       levels 1.
    2. I set CRT 0 and music 40 in the file. A menu shot showed "BEST 7" and no CRT
       (`target/shots/s048-menu-saved.png`).
    3. A patched run (since restored, checked with cmp) opened Settings and triggered
       `Activate` on SHAKE. The file got `"shake": false`, and the next clean run loaded
       it (`target/shots/s048-settings.png`).
    4. A fresh save showed "NEW BEST!" (`target/shots/s048-gameover.png`).
  - iOS simulator (`just ios-sim`, Mac muted):
    1. The first launch wrote `Library/Application Support/rix/save.json` in the app
       container.
    2. I stopped the app and set best 4242, CRT 0, music and sound 0 in the file.
    3. After a new launch, the title showed "BEST 4,242" and no CRT
       (`target/shots/s048-ios-title.png`). Then I stopped the app.
  - Android was not run on the emulator. Only clippy covers the Android path.
- Open points:
  - The lifetime stats are saved but not shown anywhere yet.
  - A run quit mid-way (app killed) adds no stats.
