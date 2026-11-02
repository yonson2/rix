# bevy_new_2d study for Rix

Clone: TheBevyFlock/bevy_new_2d (main @ `bb9fa3e`, 2026-08-06).
Template paths are relative to that repo. Rix paths are as of 2026-11-02 (Bevy 0.15 code, before [013]-[018]); line numbers drift.

## Template facts

- **Bevy version:** main targets **Bevy 0.19** (`Cargo.toml:9`, `Cargo.lock` bevy 0.19.0, commit "Update to bevy 0.19 (#489)").
  Newest version tag: `bevy-v0.18` (Bevy 0.18). The GitHub "Latest" release object is `v0.1.0` (2024-07-11), stale.
  There is no 0.20 version yet. Rix on 0.20 will be one minor version ahead of the template.
- 0.18 to 0.19 changes in the template (`git diff bevy-v0.18 HEAD`): run conditions `.and()/.or()` became `.and_then()/.or_else()`,
  `UiDebugOptions` became `GlobalUiDebugOptions`, `load_with_settings` became `load_builder().with_settings(..).load(..)`, feature `"2d"` became `["2d","ui","audio"]`.

```
src/main.rs            AppPlugin, AppSystems sets, Pause state, camera (binary-only crate)
src/asset_tracking.rs  LoadResource trait + ResourceHandles
src/audio.rs           Music / SoundEffect markers + bundle fns
src/dev_tools.rs       compiled only with feature "dev"
src/screens/{mod,splash,loading,title,gameplay}.rs   Screen state
src/menus/{mod,main,pause,settings,credits}.rs       Menu state
src/theme/{mod,palette,interaction,widget}.rs        UI builders
src/demo/{mod,level,player,movement,animation}.rs    sample game
```
- **Plugins are functions:** `pub(super) fn plugin(app: &mut App)`, one per file; parents call `app.add_plugins((a::plugin, b::plugin))`
  (`docs/design.md` "Plugin Organization", `src/screens/mod.rs:10-19`). Modules are private `mod x;`.
- **Two independent states:** `Screen {Splash, Title, Loading, Gameplay}` (`src/screens/mod.rs:22-29`) and `Menu {None, Main, Credits, Settings, Pause}`
  (`src/menus/mod.rs:21-29`). The title screen only sets `Menu::Main` on enter and `Menu::None` on exit (`src/screens/title.rs`).
- **State-scoped entities:** each root gets `DespawnOnExit(state)` (`src/demo/level.rs:43`, `src/menus/pause.rs:19`; pause overlay uses
  `DespawnOnExit(Pause(true))`, `src/screens/gameplay.rs:54`). No despawn systems, no extra setup: `init_state` is enough (`src/screens/mod.rs:11`).
  Bevy docs (latest): on by default, opt out with `#[states(scoped_entities = false)]`; `DespawnOnEnter` and `DespawnWhen::new(|transition| ..)` also exist.
- **Screen-scoped resources:** insert OnEnter, remove OnExit (`SplashTimer`, `src/screens/splash.rs:27-28,128-134`), or reset OnExit (`docs/design.md` `reset_highscore`).
- **Asset preloading:** `#[derive(Resource, Asset, Clone, Reflect)]` struct with `#[dependency] Handle<..>` fields + `FromWorld`, registered with
  `app.load_resource::<T>()` in the owning plugin (`src/demo/player.rs:18,94-122`). `LoadResource` (`src/asset_tracking.rs:12-37`) adds the struct as an asset,
  polls `is_loaded_with_dependencies` in `PreUpdate` (`:57-71`), then inserts it as a resource. `ResourceHandles::is_all_done()` gates
  `Loading -> Gameplay` (`src/screens/loading.rs:11-31`) and the Play button (`src/menus/main.rs:32-42`). Early systems use `If<Res<T>>` (`src/theme/interaction.rs:92`).
- **Pause:** `States` struct `Pause(pub bool)` + `PausableSystems` set. One line gates the set:
  `configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))))` (`src/main.rs:69-71,91-97`). Systems add `.in_set(PausableSystems)`
  (`src/demo/movement.rs:26`). P/Escape toggles; unpause OnExit(Gameplay) and OnEnter(Menu::None) (`src/screens/gameplay.rs:10-33`).
- **`AppSystems {TickTimers, RecordInput, Update}`** chained in `Update` (`src/main.rs:59-67,81-89`); used via `.in_set(AppSystems::X)` (`src/demo/animation.rs:22-29`, `src/demo/player.rs:21-26`).
- **Bundle functions:** `fn player(..) -> impl Bundle`, composed with `children![]` (`docs/design.md` "Bundle Functions", `src/demo/player.rs:30-59`, `src/demo/level.rs:33-52`).
- **Theme/widgets:** `widget::{ui_root, header, label, button, button_small}` return `impl Bundle` (`src/theme/widget.rs`). A button takes an observer
  `fn(On<Pointer<Click>>, ..)` attached with `.observe(action)` (`:93-131`). `InteractionPalette` + 4 pointer observers recolor `BackgroundColor`
  and play hover/click SFX (`src/theme/interaction.rs`). Colors in `src/theme/palette.rs`; `theme::prelude` re-exports.
- **Audio:** markers `Music` / `SoundEffect`; `music(h)` = `AudioPlayer + PlaybackSettings::LOOP + Music`, `sound_effect(h)` = `.. DESPAWN + SoundEffect`
  (`src/audio.rs:14-34`). `apply_global_volume` pushes `GlobalVolume` to live sinks (`:36-44`). Settings has master volume only (`src/menus/settings.rs:81-104`).
  Music is a child of the level root, so `DespawnOnExit` stops it (`src/demo/level.rs:44-50`).
- **Dev tools:** `#[cfg(feature = "dev")] mod dev_tools;` (`src/main.rs:9-10,51-52`): `log_transitions::<Screen>` and backquote UI-debug toggle (`src/dev_tools.rs`).
  No inspector is wired in (only recommended in `docs/tooling.md`).
- **Reflect:** all components/resources derive `Reflect` + `#[reflect(Component|Resource)]` (`src/audio.rs:14-16`, `src/screens/splash.rs:84-120`).
  No `register_type` anywhere: auto registration (`reflect_auto_register`, part of Bevy 0.20 `default_app`). Every entity gets `Name::new` (`src/main.rs:100`).
- **App setup:** `AssetMetaCheck::Never` (itch/web fix) + `fit_canvas_to_parent: true` (`src/main.rs:26-44`), `windows_subsystem = "windows"` outside dev (`:4`), `bevy_lint` tool attr (`:2`).

## Adopt in Rix (ranked)

1. **`DespawnOnExit` + a Gameplay sub-state (M).** Replaces `despawn_screen::<T>` (`src/rix/mod.rs:79,83-88`), markers `RixComponent`, `MenuComponent`
   (`src/menu/mod.rs:26,30`), `GameOverComponent` (`src/rix/game_over.rs:15-18,30`), and `despawn_screen::<Wall>` / `::<DrawnShape>` OnEnter(GameOver|Menu)
   (`src/rix/wall.rs:20-21`, `src/rix/polygon.rs:39-40`). Problem: walls and shapes must survive `InGame -> LevelTransition`, so cleanup is split and
   `spawn_walls` needs the `if !walls.is_empty()` guard (`wall.rs:104`). Explosions (`enemies/mod.rs:152-161`, `items/basic_bomb.rs:77-85`) have no marker and leak into the next screen.
   Proposal: `Screen {Loading, Title, Gameplay, GameOver}` + `#[source(Screen = Screen::Gameplay)] Phase {Playing, LevelTransition}`.
   `DespawnOnExit(Screen::Gameplay)`: walls, shapes, HUD, music. `DespawnOnExit(Phase::Playing)`: enemies, items, temp lines, countdown text, explosions.
   **Side finding:** `on_remove_enemy` (`enemies/mod.rs:141-165`) also fires for cleanup despawns: each level end or death adds +1 `kill_count` and one explosion
   per live enemy. A real kill counts twice (`:132` and `:164`). Fix this when cleanup moves.
2. **`LoadResource` collections + Loading screen instead of `AppState::Setup` (M).** Replaces `asset_server.load("fonts/main.ttf")` in `menu/mod.rs:51`,
   `rix/ui.rs:35`, `rix/game_over.rs:34`, `rix/items/mod.rs:153` (every frame, inside `display_countdown`) and `items/basic_bomb.rs:42`. Closes TODOs
   `menu/mod.rs:34` and `basic_bomb.rs:27`. Copy `src/asset_tracking.rs` as is (S). Add `UiAssets {font}`, `ItemAssets {bomb_basic}`, later `AudioAssets`.
   Gate Menu -> Gameplay on `is_all_done()` (the 2.6 s RIX flicker can hide the load). Setup's other job (`setup.rs:30-37`, `EnemySystems`/`ItemSystems`)
   is not needed: `commands.run_system_cached(spawn_ball)` works without a resource (Rix change, not template).
   From `docs/known-issues.md`: warm custom pipelines at load (crt, pixel_explosion, cartoon_explosion, glow) so the first kill does not hitch on mobile GPUs.
3. **One `PausableSystems` set (S).** Replaces 12 `.run_if(in_state(IsPaused::Running))`: `polygon.rs:37`, `wall.rs:24`, `items/mod.rs:36`, `effects.rs:56`,
   `user_input.rs:33`, `basic_bomb.rs:23`, `level.rs:34`, `setup.rs:25`, `square.rs:32`, `arrow.rs:25`, `enemies/mod.rs:32`, `ball.rs:32`.
   Keep Rix's `IsPaused` SubState (it resets on exit; better than the template's manual `unpause`). Nothing sets `IsPaused::Paused` today.
   When it does, also pause avian (`Time<Physics>`) and pause on app suspend. The template covers neither.
4. **`AppSystems` ordering (S).** `user_input.rs:25-34` is chained inside, but its event readers are unordered: `arrow.rs:24` (InputPosition),
   `polygon.rs:31-34` (DrawLine/BuildPolygon/ClearLines), `wall.rs:24` (WallEvent). Input in `RecordInput`, the rest in `Update`: no random 1-frame lag.
   `TickTimers` fits `despawn_after` (`level.rs:150`) and the `utils.rs` fade/color tickers.
5. **Split `LevelInfo` into scoped resources (S-M).** `setup.rs:39-107` mixes run stats, level progress and RNG (`#[deref] rng`). Each `level_info.rng.*`
   (`ball.rs:91` per bounce, `square.rs:120`, `level.rs:91` 100k samples) marks `LevelInfo` changed, so `update_ui` (`ui.rs:22`) reformats 3 strings per collision
   and `handle_current_level` (`setup.rs:114`) re-runs. Area sampling drains the seeded RNG, so the seed (`setup.rs:96`, `level.rs:63`) does not reproduce a level.
   Template pattern: reset/insert on state enter (`splash.rs:27-28`; `docs/design.md`). Proposal: `RunStats {points, kill_count, seed}` reset OnEnter(Gameplay)
   (GameOver still reads it), `LevelProgress {level, target, current}`, `GameRng`, separate sampling RNG. Removes `reset()` calls in `game_over.rs:110,118`.
6. **Function plugins + private modules (S-M).** 14 `struct XPlugin; impl Plugin` blocks. `lib.rs` and `rix/mod.rs` make every module `pub`, which hides dead code:
   `wall.rs:283 get_window_inner_corners`, `camera.rs:37 get_cursor_world_position`, `utils.rs:191 generate_item_position`, `polygon.rs:392 area`,
   `PlayableArea` (`wall.rs:50`, written, never read). Keep `lib.rs` (mobile), expose one `AppPlugin` that merges `BasePlugin` + `RixPlugin` (`main.rs:5-8`).
7. **Bundle functions + `theme/` widgets (M).** `ui.rs:56-96` repeats `TextFont {font.clone(), size}` 3x; `game_over.rs:50-96` 5x (only the title uses the game font).
   Add `theme::widget::{ui_root, header, label}` that take `&UiAssets`; move `colors.rs` to `theme/palette.rs`. Turn spawns into `fn ball(..) -> impl Bundle`
   (`ball.rs:57-72`, `square.rs:66-84`, `arrow.rs:66-82`, `basic_bomb.rs:43-61`) with `DespawnOnExit` inside. Use `widget::button` only when a pause/settings button comes.
8. **`audio.rs` (S, when [043] lands).** Copy markers, bundle fns, `apply_global_volume`. Music as child of the Gameplay root.
   Songs are `.mp3`; the `audio` collection is `bevy_audio + vorbis` only. Add `bevy/mp3` or convert to `.ogg` (smaller, like the template).
9. **`dev_tools.rs` behind `dev` (S).** `log_transitions::<S>` for each Rix state + UI-debug toggle. Add the `tracing` `release_max_level_warn` feature
   (`Cargo.toml:14-17`) so hot-path `info!` calls (`level.rs:119,132`, `enemies/mod.rs:128`, `items/mod.rs:81,237`, `user_input.rs:239,243,321`) leave release/mobile builds.
10. **`Reflect` + `Name` on Rix types (S, low).** Useful only with an inspector or BRP. No `register_type` needed.

## Skip or adapt

- **Binary-only crate** (`src/main.rs` holds all modules): skip. Rix needs `lib.rs` for the Android `cdylib` / iOS `staticlib` entry.
- **Splash screen** with Bevy logo (`src/screens/splash.rs`): skip. The RIX flicker (`menu/mod.rs`) is the intro; the fade-timer code is reusable.
- **Button main menu with Exit** (`src/menus/main.rs`): "tap anywhere" fits touch. If menus come, hide Exit on iOS/Android as the template hides it on wasm (`:16-28`).
- **Keyboard-only flow** (P/Escape pause `src/screens/gameplay.rs:14-27`, Escape back in menus): adapt. Mobile needs an on-screen pause button,
  Android back, and pause on `AppLifecycle` suspend. The template has no lifecycle handling.
- **Theme visuals** (`src/theme/interaction.rs` flat `BackgroundColor`): adapt. Rix neon uses HDR colors > 1.0 (`colors.rs`) + bloom. Rix's CRT node sits between
  `Tonemapping` and `EndMainPassPostProcessing` (`crt.rs:70-77`); Bevy UI likely renders after that, so UI nodes get no bloom/CRT (verify on 0.20).
  That is why the menu uses world-space `Text2d`. Keep neon parts world-space; use UI widgets only for plain HUD/menus.
- **Demo gameplay** (`src/demo/movement.rs` transform movement + `ScreenWrap`, sprite-atlas animation, `rand`): skip. Rix uses avian2d, procedural meshes, seeded `fastrand`.
- **Global `Pause(bool)`**: adapt to Rix's SubState (Adopt 3).
- **"No 3rd-party deps"** (`docs/design.md`): not a Rix goal (avian2d, lyon_tessellation per D2).
- **GitHub Pages deploy, cargo-generate files** (`cargo-generate.toml`, `post-generate.rhai`, `*.template`), `.idea/`: skip. `.vscode/` snippets: optional.
- **`default-features = false, features = ["2d","ui","audio"]`** (`Cargo.toml:9`): adapt. Bevy 0.20 `2d` lacks what `camera.rs` uses: `tonemapping_luts`
  (TonyMcMapface, `camera.rs:25`), `bevy_anti_alias` + `smaa_luts` (Smaa, `camera.rs:28`) live only in the 3d collections (Bevy `docs/cargo_features.md`).
  Add them, `mp3` if kept, and check that Android (`android-game-activity`) still builds with defaults off.

## Tooling worth copying

- **Features** (`Cargo.toml:22-51`): `dev` = `bevy/dynamic_linking`, `bevy_dev_tools`, `bevy_ui_debug`, `track_location`; `dev_native` adds `file_watcher`, `embedded_watcher`.
  Rix has dev/dev_native without the 3 debug features. Template uses `default = ["dev_native"]`, stripped by `[package.metadata.bevy_cli.release|web] default-features = false`.
  **Caution:** with that default, plain `cargo build --release` (Xcode/Gradle scripts) links Bevy dynamically and breaks mobile.
  Keep Rix's no-default + `just dev`, or pass `--no-default-features` in every mobile script.
- **Profiles** (`Cargo.toml:78-115`): Rix already matches dev/release. Add `[profile.dev.package.wgpu-types] debug-assertions = false`, rename Rix `wasm-release`
  to `web-release` (bevy_cli uses it automatically, `:97`), add `profile.ci` (inherits dev, opt 0, `debug = "line-tables-only"`, `codegen-units = 4`, deps opt 0).
- **Lints** (`Cargo.toml:54-73`, `clippy.toml`): `too_many_arguments` + `type_complexity` = allow (drops Rix `#[allow]`s at `ui.rs:100`, `items/mod.rs:133`,
  `user_input.rs:166`); `nonstandard_macro_braces = "warn"` + `clippy.toml` forces `children![..]`; `unexpected_cfgs` check-cfg `bevy_lint`; `[package.metadata.bevy_lint]`.
- **CI** (`.github/workflows/ci.yaml`): push/PR to main, concurrency cancel, pinned `nightly-2026-04-16`, `RUSTFLAGS=-Dwarnings -Zshare-generics=y -Zthreads=0`.
  Jobs: fmt, `cargo doc`, clippy (`--all-targets --all-features --profile ci`), `bevy_lint` (action `TheBevyFlock/bevy_cli/bevy_lint@main`), tests with cranelift,
  `check-web` (wasm32, `--no-default-features --features dev`, getrandom `wasm_js` cfg). `Swatinem/rust-cache` shared keys, saved only on main.
  Ubuntu deps: `libasound2-dev libudev-dev libwayland-dev`. Rix has no `.github/` and no tests; first test targets: `is_point_in_polygon`,
  `line_segments_intersect`, `calculate_enemy_count`, `Points` Display. Add jobs the template lacks: `cargo check` for `aarch64-apple-ios` (macos runner) and `aarch64-linux-android` (NDK).
- **Release** (`.github/workflows/release.yaml.template`): `workflow_dispatch` with `version` + per-platform booleans (`:3-42`), optional CD on push (`:102-106`).
  Matrix web/linux/windows/macos (`:151-172`). `cargo-binstall` then `bevy_cli 0.1.0-alpha.2` from git (`:245-251`). Web: `wasm-bindgen-cli` + `wasm-opt` +
  `bevy build --locked --release --yes web --bundle` (`:253-259`). Native: `bevy build --release --target`; macOS universal `lipo`, generated `Info.plist`,
  `.dmg` via `hdiutil` (`:261-321`), **no codesign/notarize**. GitHub release upload via `svenstaro/upload-release-action`.
  **itch.io** (`:354-384`): secret `BUTLER_CREDENTIALS` (itch API key), env `itch_page: user/project`, butler from `broth.itch.zone`,
  `butler push --fix-permissions --userversion`, one channel per platform (`web`, `linux`, `windows`, `macos`). After the first push: itch "Kind of project = HTML",
  mark the web upload playable (`docs/workflows.md`). The repo's own `release.yaml` adds `libs_path: lib`, rpath `$ORIGIN` / `@loader_path`, and copies
  `lib/<platform>/*` into the app (`release.yaml:60,191-198,278-281`): use it for Steamworks `steam_api` libs. No Steam/GOG upload step exists.
- **Web build:** `AssetMetaCheck::Never` + `fit_canvas_to_parent` (`src/main.rs:26-44`), `web-release` (`opt-level = "s"`, `strip = "debuginfo"`), `bevy run web`.
  Rix risks: `crt.rs:259` hardcodes `Rgba16Float`; WebGL2 16-byte uniform padding is commented out (`crt.rs:287-289`); window fixed 625x625 (`base.rs:13-17`);
  wasm seeding of global `fastrand::u64` (`setup.rs:96`) may need fastrand's `js` feature (verify).
- **bevy_cli:** `bevy run`, `bevy run web`, `bevy run --release [web]` (`README.md`, `.vscode/tasks.json`), `bevy new my_game --template 2d`. No mobile commands used.
- **Hot reload:** assets via `dev_native`; system hot-patching via `BEVY_ASSET_ROOT='.' dx serve --hot-patch --features "bevy/hotpatching"` (`README.md`).
  Inspector: `bevy-inspector-egui` only recommended (`docs/tooling.md`).
- **`.cargo/config_fast_builds.toml`:** linker notes; on Apple Silicon keep default ld64. Optional nightly `-Zshare-generics` / `-Zthreads`.
- **Mobile: none.** No iOS/Android target, feature, CI job, release step or doc (grep finds `android` only in `Cargo.lock`, as transitive Bevy deps).
  Rix keeps its own `rix.xcodeproj` and Android pipeline.

## Open questions

- Gameplay sub-state (`Phase`) or 0.20 `DespawnWhen` for walls/shapes?
- Keep `.mp3` (+`bevy/mp3`) or convert songs to `.ogg`?
- Is the per-enemy explosion at level end intended?
