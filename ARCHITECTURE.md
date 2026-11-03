# Architecture

This file describes Rix as it IS. The target shape lives in the issues ([026] and M3).
Update this file in the same commit as a structural change (modules, plugins, states,
events, dependencies). Names are given without links, so search for them (`grep`, or
symbol search in the editor).

## Bird's-eye view

Rix is a Qix-like game on Bevy 0.20 with avian2d 0.8 physics. The player presses on a
wall and drags a line across the field. Each part of the line is a short-lived entity with
a collider. If an enemy touches the line, the player dies. The free region (the unclaimed
part, one polygon) starts as the box inside the walls. A line runs from its boundary (a
wall or a claimed edge) to its boundary and cuts it in two ([074], D19). The side
without enemies becomes claimed area: a filled shape with a static collider. When the
claimed part of the field reaches 75 %, the level ends, the shapes fade out and the next
level starts with more enemies.

One `App` holds everything. `rix::app()` adds the plugins of `base` (Bevy, physics,
camera, CRT), `screens` (the `Screen` state, title, settings, seed entry and game over),
`game` (all game logic), `audio`, `haptics`, `save` and `share`, and `dev_tools` with the `dev_tools` feature. The desktop binary and the mobile
library (`mobile/`) both run it.

Each module has its own plugin, `pub(super) fn plugin(app: &mut App)`, added by its
parent module ([026], the bevy_new_2d pattern). Below, "plugin" means that function.

## Code map

### Crate root (`src/`)

- `main.rs`: the desktop binary; runs `rix::app()`.
- `lib.rs`: the module list and `app()`. The binary, the mobile crate, the examples and the
  tests use the library. Only `app`, `crt` and `theme` are public (the examples use
  `crt::PostProcessSettings` and `theme::palette`); the other modules are private, so
  rustc reports dead code ([035]).
- `base`: plugin. `DefaultPlugins` with one resizable 450 x 800 window (the default field),
  avian `PhysicsPlugins` with `PhysicsInterpolationPlugin::interpolate_all()` ([076]),
  the camera and CRT plugins. It makes
  every `Collider` require `CollisionEventsEnabled`. In a shot run it leaves out
  `WinitPlugin` ([064]).
- `camera`: the plugin spawns the one `Camera2d` in `PreStartup` with `Hdr`,
  `Tonemapping::TonyMcMapface`, `Bloom`, `Smaa` (no MSAA) and `PostProcessSettings`.
  `fit_camera` sets its `ScalingMode::AutoMin` to the `Field` when the field changes.
- `crt`: the CRT post-process. `PostProcessSettings` is extracted to the
  render world. The render system `post_process` runs in `Core2dSystems::PostProcess`,
  after tonemapping and SMAA, so it is the last effect. Shader: `crt_shader.wesl`.
  `PostProcessSettings::shake` offsets the picture (the screen shake, [040]), so input
  and picking never see the shake.
- `shape`: `Shape` (a lyon `Path`, an optional fill and an optional `Stroke`) and its
  plugin. When a `Shape` changes, `mesh_shapes` tessellates it into one mesh with
  vertex colors, drawn with one shared white `ColorMaterial` (`ShapeMaterial`). Helpers:
  `polygon`, `rounded_polygon`, `rounded_rectangle`. This replaces bevy_prototype_lyon
  (D2).
- `asset_tracking`: `LoadResource` (`app.load_resource::<T>()`) and `ResourceHandles`,
  from bevy_new_2d ([027]). A collection is a `Resource + Asset` struct of `#[dependency]`
  handles built in `FromWorld`; it becomes a resource once every handle has loaded.
- `audio` ([041]): music and sound effects, after bevy_new_2d. `Volumes` (music, sfx; 0 to
  1, set from the settings, [048]); `AudioAssets` (a `LoadResource` collection); `sound_effect(handle)` plays
  once at the sfx volume (an `Add<SoundEffect>` observer sets the start volume).
  `MusicTrack` names the track for the screen (menu on `Title`, game on `Gameplay`);
  `switch_music` and `fade_music` crossfade it in 1 s (real time). Sinks pause on
  `AppLifecycle::Suspended` and play on `Running`. A button `Activate` plays the tap.
  Sound effects ([042]): game code triggers `Sfx` (claim, eat, bomb, bounce; bounces at
  most one per 80 ms); death and level clear play on their state entry; `hum_while_drawing`
  plays the line start and loops the hum while a line exists. A shot run sets both volumes
  to 0 (in `save::apply_settings`). Sources and licences: `audio-src/sources.md`.
- `haptics` ([081]): touch feedback. The `Sfx` event asks for a pulse (claim light, eat and
  bomb medium), the death asks for a heavy one; `Pending` keeps the strongest of a frame and
  `pulse` (main thread, `NonSendMarker`) plays it: `MainActivity.haptic` over JNI on Android
  (`performHapticFeedback`, which follows the system touch-feedback setting),
  `UIImpactFeedbackGenerator` on iOS, a log line elsewhere. The settings can turn it off.
- `save` ([048]): the `Save` resource: `best` score, lifetime `Stats` (runs, claims, kills,
  levels cleared, seconds of play) and `Settings` (music, sound and CRT in percent, shake,
  haptics, claim flash). JSON (`serde_json`) in the app data directory: `HOME/Library/
  Application Support/rix` on macOS and iOS (iOS gives each app its own `HOME`), the
  Android app's internal data path, `APPDATA` or `XDG_DATA_HOME`; none on the web yet. It
  loads in the plugin (no file or a bad one: defaults) and `write` (through a temporary
  file) runs whenever `Save` changes, also at the start, so the file exists after the first
  launch. `RunStats` counts the claims (`DrawPolygon`) and the play time of the run;
  `record_run` (pure, tested) adds a finished run at the game over. `apply_settings` sets
  `Volumes` and the CRT `intensity`; `fx` reads `shake` and `flashing`, `haptics` reads
  `haptics`. A shot run has no save file unless `RIX_SAVE` names one.
- `share` ([067]): the events `Share(text)` and `Paste`, answered by `Pasted(text)`. `run`
  (main thread, `NonSendMarker`) does the work. On phones: the share sheet and the system
  clipboard, `MainActivity.share` and `pasteText` over JNI on Android,
  `UIActivityViewController` and `UIPasteboard` on iOS. Elsewhere Bevy's `Clipboard`: a
  share is a copy (`SHEET` is false); the desktop gets the `system_clipboard` feature
  (`arboard`, desktop targets only), the web reads the clipboard async.
- `theme`: the shared look. `FontAssets` (the font, a `LoadResource` collection).
  `theme::palette`: color constants. `NEON_*` values are above 1.0, so they bloom in HDR.
  `theme::widget` ([036]): `button(label, font, y)` (and `button_at` with a position and
  a size), a world-space neon box with a
  `Text2d` label (`NeonButton`), so the game camera draws it with bloom and CRT (Bevy UI
  draws after the CRT pass). `press_buttons` (`Update`) tracks a touch or a left click: a
  press and a release on the same button trigger the entity event `Activate` on it; the
  screen adds an observer (`.observe(..)`). The pressed button shrinks a little.
- `common`: `Transition`, a timer component (menu title fade, enemy spawn animation).
- `utils`: `ColorTransition` and `update_sprite_color` / `update_mesh_color` (color
  pulses), `FadeShape` and `update_shape_fade` (claimed shapes fade out between levels),
  `generate_enemy_position` (a spot in the field where a collider meets no other, so not
  in a claim either; `None` after 64 tries, [032]).
- `screens`: the `Screen` state (`Loading`, `Title`, `Settings`, `Seed`, `Gameplay`,
  `Restart`, `GameOver`, [026]). `Restart` lasts one frame: the pause menu's RESTART goes
  through it, because setting `Gameplay` again keeps its sub-states ([073]).
  - `loading`: the first screen (blank); it goes to `Title` when
    `ResourceHandles::is_all_done` ([027]).
  - `title`: the title screen in `Screen::Title`: the RIX logo (it flickers in, then
    breathes), the best score (when there is one), PLAY (goes to `Screen::Gameplay`),
    PLAY SEED (`Screen::Seed`), SETTINGS and, on desktop only, QUIT (`AppExit`).
  - `seed` ([067]): a keypad of the 32 code characters, DEL, PASTE (`share::Paste`; the
    `Pasted` observer takes the code with `seed::find`), PLAY and BACK; a keyboard types
    too. The typed code (`Typed`) shows grouped, gray until it reads as a seed. PLAY calls
    `new_run` with the code's seed and goes to `Screen::Gameplay`.
  - `settings` ([048]): one button per setting; a tap steps it (percent down by 20 and
    from 0 back to 100, switches on and off) and `relabel` rewrites the labels whenever
    `Save` changes. `spawn_panel` spawns the title, the rows and BACK as children of a
    root the caller gives (lifetime and height): the settings screen (BACK goes to the
    title) and the pause menu ([073]) use it.
  - `game_over`: the game over screen ([038]), world `Text2d` and `widget::button`s:
    "YOU DIED!", the score, level and kills, the seed code, and "NEW BEST!" or the best
    score (its setup calls `save::record_run`); RETRY starts a new run, MENU goes to the
    title with a fresh run (so PLAY starts it), SHARE (COPY off phones, then COPIED) sends
    "Rix seed CODE" with `share::Share`.
- `dev_tools` (feature `dev_tools` only): plugin. `RIX_SEED` sets the seed of
  the first run. `RIX_SHOT` runs a scripted game (or the menu with `RIX_SHOT_SCENE=menu`,
  or a level clear, death and retry with `RIX_SHOT_SCENE=flow`) without a window, saves
  one picture and exits; `RIX_SHOT_SCENE=replay` runs the game line without the renderer
  and exits after the state hash ([075]). See "Dev tools" below.

### Game (`src/game/`)

- `mod.rs`: the plugin adds the sub-states, the system sets (`AppSystems`,
  `PausableSystems`) and the plugins below. `GameLayer` and
  `get_player_layer` / `get_enemy_layer` / `get_item_layer` set the collision layers
  (`shooter` builds the bullet layer).
- `state`: the sub-states of `Screen::Gameplay`: `Phase` (`Playing`, `LevelTransition`),
  then `IsPaused` (`Running`, `Paused`, `Dying`) in `Phase::Playing`, `PausePanel`
  (`Menu`, `Settings`) in `IsPaused::Paused` ([073]) and `TransitionState` in
  `Phase::LevelTransition` ([026]). Entities carry `DespawnOnExit` of the state they belong to (see Invariants).
- `setup`: the plugin adds the `events` and `effects` plugins. Three resources hold the game state ([028]):
  `Run` (seed, level, `Points`, kill count), `LevelProgress` (target 0.75, claimed
  fraction `current`) and `GameRng` (the seeded `fastrand::Rng`). `new_run` inserts all
  three for a seed; `random_seed` gives 40 bits (an eight-character code).
  `handle_current_level` starts the level transition when the target is reached.
- `seed` ([067]): seed codes, pure and tested. `code` writes a seed in Crockford base32 in
  groups of four (at least eight characters); `parse` reads one back (no dashes or spaces,
  any case, I and L as 1, O as 0); `find` takes the code out of a pasted message.
- `events`: the messages between systems (`DrawLine`, `ClearLines`, `BuildPolygon`,
  `DrawPolygon`, `Die`, `ActivateItem`, `ActivateBasicBomb`, `ActivateAutoLine`,
  `IncreasePoints`, `UpdatePoints`, `InputPosition`) and the `DespawnEnemy` event (an observer trigger).
- `user_input`: `handle_input_events` turns each `WindowEvent` (touch, click, cursor
  move, in arrival order) into a step of the one pointer in `InputState` (press, move,
  release; world coordinates, [077]). It drops a press on a `NeonButton` (the HUD's pause
  button), so that press, its moves and its release draw nothing ([073]). `handle_input` runs the line logic once per step,
  keeps the line being drawn in `UserPath` and sends the line messages. An
  `ActivateAutoLine` puts a `Shot` step first: a move to `auto_line::shot_target` that
  claims like a move onto the wall, or does nothing if it would cross the line ([046]).
  Pure helpers with tests: `start_point`, `would_create_intersection`.
- `territory` ([074]): `FreeRegion`, the free region as one simple counter-clockwise
  polygon, reset to the box on `OnEnter(Phase::Playing)`. Pure geometry with tests:
  `locate` (nearest boundary point), `split` (cut by a path from boundary to boundary),
  `line_end` (the input rule), `signed_area`, `claimed_fraction`. The tests include the
  [029] fuzz (2,000 seeded claims, every cut checked).
- `wall`: four `Wall` entities with static colliders, spawned
  on `OnEnter(Screen::Gameplay)` after `fit_field`. The `Field` resource (walls included,
  D21, [080]) is the playfield: 450 wide, and `Field::for_view` takes the height from the
  camera's view shape at the start of a run (clamped from 600 to 1050; the default is
  450 x 800). `WALL_THICKNESS` (35) and `WALL_THICKNESS_TOP` (65, room for the HUD, and
  below a phone's notch or Dynamic Island). `Field::inner_corners` gives the
  `WallCorners`.
  `WallEvent` makes the walls glow while a line is drawn.
- `polygon`: the line segments (`TempLineSegment`), the claim (`split` ->
  `get_correct_polygon`) and the claimed shapes (`DrawnShape` with the
  `Territory` marker). `is_point_in_polygon` is the one
  point test the game uses.
- `level`: `calculate_area_progress` (claimed fraction), points
  (`ENEMY_POINTS`, `ITEM_POINTS`, doubled per level), the level transition with its
  "LEVEL N" `Banner` ([038]), and `DespawnAfter` (a timer that despawns its entity).
- `death` ([038]): `IsPaused::Dying` stops virtual and physics time, `flash_line`
  (real time) flashes the `TempLineSegment`s red and white for 1 s, then goes to
  `Screen::GameOver`.
- `enemies`: `Enemy`, `spawn_enemies` (picks each kind from the spawn table and runs its
  spawn system with `run_system_cached`), death on line contact, and the kill observer
  `on_despawn_enemy`. A spawn with no free spot is skipped. A kill leaves a `Corpse`
  ([039]): a copy of the enemy's mesh or shape at z 1.5 that pops white, then shrinks
  away in 0.4 s (`animate_corpses`, `Update`, not pausable, so it also plays during the
  level transition; it lives for the screen).
  - `ball`: `Ball`, moves at a fixed speed, gets a random turn on each hit.
  - `square`: `Square`, falls up or down (`GravityScale`), pulses and teleports; with no
    free spot to teleport to, it dies (a kill).
  - `arrow`: `Arrow`, turns toward the finger (`InputPosition`) and glows while a line is
    drawn.
  - `splitter` ([044]): `Splitter::{Queen, Soldier}`, from level 2. Both `wander`: straight
    runs at a constant speed, a random turn every 1 to 2.5 s, bounces off the walls. A
    Queen kill spawns a `Hatch`; 0.25 s later (so the claim that ate her has its collider)
    it becomes two faster Soldiers at free spots. Soldiers do not split.
  - `shooter` ([045]): `Shooter`, a fixed turret from level 3 (dynamic, all axes locked, so
    the line meets it) that turns toward the finger. While a line is drawn its `Gun` fires
    a slow `Bullet` (a sensor on `GameLayer::Bullet`) 0.5 s in, then every 1 s;
    `bullet_hits` kills on the line (`Die`) and drops the bullet on a wall or claim.
    Bullets are not enemies: claims do not eat them.
  - `rng`: `calculate_enemy_count` (per level) and the spawn table: `EnemyKind`,
    `SpawnRule` (kind, level band, weight), `SPAWN_TABLE` and `pick_kind` (a weighted
    pick that draws `u64`, so a seed picks the same on 32-bit targets). Pure, tested.
- `items`: `Item`. Items spawn on level start (one coin flip) and during play
  (`ItemTimer`: every 10 s of play, a 50 % roll, [033]); `pick_item` picks evenly among
  the items of the level (`ITEM_LEVELS`: bomb from 1, multiplier 2, time stop 3,
  auto-line 4; with one item it draws nothing, so level 1 replays stay the same, [046]).
  The bomb is a glow image; the others are a token (`spawn_token`: a dark rounded square
  with the module's `icon`). Items blink before they despawn (`blink_on`); the bomb also
  shows a countdown. An item activates when a line touches it or a claim covers it (a
  claim sends each item's own kind). An item fires once even when several segments touch
  it in one tick, and each bomb of a tick takes another enemy ([083]). The effects label
  under the HUD bar shows the timed effects and their seconds left ("x2 7   STOP 3").
  - `basic_bomb`: removes one random enemy, with an explosion. `ItemAssets` holds its
    image.
  - `multiplier` (a gem): the `ScoreBoost` resource exists for 10 s; while it does,
    `level` counts kill and claim points `BOOST_FACTOR` (2) times.
  - `time_stop` (an hourglass): the `TimeStop` resource exists for 4 s. `freeze` gives
    each enemy and bullet `Frozen` (its velocity and own `LockedAxes`), zero velocity and
    `LockedAxes::ALL_LOCKED`; at the end `tick_stop` gives them back. The systems that move
    enemies or make them act run on `time_flows` (arrow chase, splitter wander, shooter
    aim and fire); a square shows in full and neither fades nor teleports. Frozen bodies
    stay dynamic, so the line still meets them and a claim still eats them.
  - `auto_line` (an arrow): sends `ActivateAutoLine`; `shot_target` (pure, tested) is the
    point on the line's heading (its last 40 units) past the free region.
- `effects`: the `Material2d` types: `ExplosionMaterial` (bomb),
  `PixelExplosionMaterial` (removed enemy), `GlowMaterial` (the bomb item: HDR glow and a
  pulsing `outline`, [040]).
- `fx` ([040]): juice that only draws, in `Update`. A new claimed piece carries `Flash`
  (white to its fill in 0.3 s; the level clear's fade keeps its alpha). The `Shake` event
  adds trauma (death 0.8, bomb 0.6); `shake_screen` (real time, so it runs while dying)
  writes `PostProcessSettings::shake`. The `Burst` event throws `Spark` sprites: a ball
  bounce (4, away from the wall) and a kill (16, the enemy's colour, HDR). Sparks draw from
  their own `FxRng`, never `GameRng`, so a seed replays the same game.
- `ui`: the in-game HUD on the top wall ([037]), world `Text2d`, `Shape` and sprites: a
  level badge on the left, the pause button ("II", a `widget::button_at`, [073]) right of
  it, the score on the right, and a bar toward the level target along the wall's inner
  edge; the middle stays free for a camera cutout. The pause button pauses only in
  `IsPaused::Running`. `count_up`
  (`Update`, draws only) moves the shown score and bar toward `Run::points` and
  `LevelProgress` in about half a second and writes only on change; the level text
  redraws only when `Run` or `LevelProgress` changes ([028]).
- `pause`: the pause triggers, the dim sprite (`IsPaused::Paused`) and the pause menu
  ([073], `PausePanel::Menu`): "PAUSED", the run's seed code, and RESUME, RESTART (a new
  run through `Screen::Restart`), SETTINGS (`PausePanel::Settings`: the settings panel,
  BACK returns to the menu) and MENU (a new run, the title). Each panel is the children
  of a root above the dim sprite, with `DespawnOnExit` of its panel. On Android, `pause::back` exports
  the JNI function `MainActivity.onBackInPlay` ([079]): an atomic tells the Java UI thread
  whether a run is in `IsPaused::Running`; if so, Back marks it and a system pauses.

### Shaders (`assets/shaders/`, WESL since [018])

- `crt_shader.wesl`: the CRT pass (`crt`).
- `cartoon_explosion.wesl`: `ExplosionMaterial`.
- `pixel_explosion.wesl`: `PixelExplosionMaterial`.
- `glow_material.wesl`: `GlowMaterial`; its outline pulses with `globals.time`.
- `flame_material.wesl`: only the `flame_shader` example uses it.

### Outside `src/`

- `mobile/`: the `rix_mobile` crate (a workspace member): a `cdylib` whose `#[bevy_main]`
  runs `rix::app()`, with Bevy's `android-game-activity` feature. `mobile/android/` is the
  Gradle project (GameActivity, `top.cosas.rix`, version from the root `Cargo.toml`);
  `just android` puts the library into its `jniLibs` ([021]). `MainActivity` hides the
  system bars (sticky immersive), excludes the whole window from the side back gesture,
  and asks the game on Back: a running game pauses, anywhere else the task moves to the
  background, as Home does ([023], [079]). Its `haptic(int)` plays the pulses of
  `src/haptics.rs` ([081]); `share(String)` opens the share sheet and `pasteText()` reads
  the clipboard (`src/share.rs`, [067]).
- `mobile/ios/`: the Xcode project (`Rix.xcodeproj`, `top.cosas.rix`, iOS 15.0). Its "Rust"
  build phase (`build_rust_deps.sh`) builds the root `rix` binary for the iOS target, and
  the app bundles the root `assets/` folder. `just ios-sim` passes the version from
  `Cargo.toml` ([020]). The window (`base.rs`) defers the system gestures on every screen
  edge and hides the status bar and the home indicator ([020], [079]).
- `examples/`: one example per shader (`glow_material`, `cartoon_explosion`,
  `pixel_explosion`, `flame_shader`).
- `assets/`: the font (`fonts/main.ttf`), the bomb image, the shaders and the sounds
  (`audio/sfx/`, `audio/music/`, Vorbis `.ogg`). Licences: [011], `audio-src/sources.md`.
- `issues/`: the tracker. `tracker/`: the Astro issue board that reads it ([004]).
- `.github/workflows/ci.yml`: CI (fmt, clippy, tests) on pushes and PRs to `main` ([008]).
- `docs/research/`: studies that issues link to. `docs/design/`: feature specs.

## State machine

```
Screen:  Loading --assets loaded--> Title --PLAY--> Gameplay --line touched, 1 s--> GameOver --RETRY--> Gameplay (new run)
                                        ^                                                       |
                                        +------------------- MENU (new run) -------------------+
                                                      |
Phase (in Gameplay):  Playing --75 % claimed--> LevelTransition (Setup -> Loading)
                         ^                              |
                         +---- no claimed shape left ---+
```

- `IsPaused` exists only in `Phase::Playing`. `pause` enters `Paused` when the app suspends, the
  window loses focus, or Escape or Android Back is pressed, and stops `Time<Virtual>` and `Time<Physics>`;
  RESUME or Escape goes back to `Running`; a tap elsewhere does nothing ([073]). Both switches drop the
  pending pointer messages (`MouseButtonInput`, `TouchInput`, `WindowEvent`), and leaving `Paused` cancels a line in progress ([034]). The
  gameplay systems are in the `PausableSystems` set, which runs only in
  `IsPaused::Running`, so they stop in every other state.
- Gameplay runs in `FixedUpdate` (64 ticks per second, [072]): input, the line and the
  claim, enemy and item behaviour, collision reactions, gameplay timers (`DespawnAfter`),
  points and progress. avian steps in `FixedPostUpdate` right after. Each rigid body's
  `Transform` is interpolated between the last two ticks for drawing and put back to the
  tick's value in `FixedFirst`, so gameplay reads stepped values and bodies move evenly at
  60, 90 or 120 frames per second ([076]). `Update` keeps what
  only draws: colour transitions, the arrow stroke, item blink and countdown, wall glow,
  explosion time, the HUD, and the level-transition fade. So a seed replays at any frame
  rate (`just replay`). State changes (level clear, death) still apply at the next frame,
  so a low frame rate can run one more tick before them.
- System order ([026]): in both schedules the sets `AppSystems::TickTimers`,
  `RecordInput` and `Update` run in that order (`src/game/mod.rs`). `user_input` records
  input; the systems that read its messages are in `AppSystems::Update`, so they see the
  input of the same tick. `FixedUpdate` runs on one thread (`SingleThreadedExecutor`), so
  all its systems run in one fixed order, also the six that share `GameRng` (claim,
  bomb, ball chaos, square teleport, splitter wander and hatch).
- `Phase::LevelTransition`: `start_transition` gives each `DrawnShape` a `FadeShape`,
  spawns the "LEVEL N" banner (1.2 s: it pops in, holds, fades) and goes to `Loading`.
  When the last shape and the banner are done, `handle_transition_loading` raises the
  level, reseeds the RNG and goes back to `Phase::Playing`.
- A line hit enters `IsPaused::Dying` (in `Phase::Playing`, so the frozen level stays on
  screen) and, after 1 s, `Screen::GameOver`. RETRY and MENU call `new_run` with a random
  seed.

## Main data flow: one line

1. `handle_input_events` reads the `WindowEvent` stream (touches, clicks and cursor
   moves in arrival order) and queues the pointer steps of this tick in `InputState`, so
   a press, its moves and its release in one tick stay apart ([077]).
2. `handle_input`: a press outside the free region (on a wall or a claim) or within
   17.5 units of its boundary starts a path in `UserPath` at the nearest boundary point
   (`start_point`) and sends `WallEvent::Bloom`. Until the finger leaves that zone, the
   start follows it. Each move in open space sends `DrawLine`. A move that crosses the
   path cancels it (`ClearLines`, `WallEvent::Reset`). A release cancels it too. While
   drawing it sends one `InputPosition` per tick, the latest position (the arrow follows
   it).
3. `handle_draw_line` spawns one `TempLineSegment` per `DrawLine`: a sprite with a
   rectangle collider on the player layer, with `DespawnOnExit(Phase::Playing)`.
4. The input rule (`line_end`, [029]): a move whose segment comes within 1 unit of the
   free boundary (sampled every 0.5 units) or crosses it ends the path. The path keeps
   that point and its snap onto the boundary, and `handle_input` sends `ClearLines` and
   `BuildPolygon(path)`. So a path touches the boundary only at its two ends.
5. `handle_build_polygon`: `split` cuts `FreeRegion` with the path into two
   counter-clockwise sides (a side with no area drops the line). `get_correct_polygon`
   claims the side with no enemy inside. If both have enemies, it claims the first one,
   and the enemies inside it are eaten (`DespawnEnemy`). Items inside the claimed side get
   `ActivateItem`. The other side becomes the new `FreeRegion`, and it sends
   `DrawPolygon`.
6. `handle_draw_polygon` spawns the claimed shape: `DrawnShape`, `Territory`, a filled
   `Shape`, `RigidBody::Static` and a trimesh collider of the shape's fill triangles
   (`territory_collider`, the same lyon fill as the mesh, [024]).
7. `calculate_area_progress` runs when `FreeRegion` changes. It sets
   `LevelProgress.current` to 1 - free area / box area (shoelace, exact). A gain above 1 %
   sends `UpdatePoints`. `handle_update_points` adds points (times `2^(level - 1)`).
8. `handle_current_level` sees `current >= target` and goes to `Phase::LevelTransition`.

Death: `handle_enemy_collision` reads avian `CollisionStart`. An `Enemy` with a
`TempLineSegment` sends `Die`, and `handle_die` enters `IsPaused::Dying` (`death`).

Enemy kill: the claim (eaten) and the bomb trigger `DespawnEnemy`. Its observer
`on_despawn_enemy` despawns the enemy, counts one kill, spawns a pixel explosion and sends
`IncreasePoints`. Enemies that leave with `Phase::Playing` (level clear, death) are no
kill and leave no explosion ([030]).

## Cross-cutting concerns

**Coordinates.** World origin is the field center, y up. The field (`wall::Field`, D21,
[080]) is 450 world units wide and takes the screen shape at the start of each run, so a
phone shows no letterbox; a seed keeps its RNG draws, and positions follow the field. The
camera's `ScalingMode::AutoMin` fits the whole field into any window or image and
letterboxes the rest (a field outside the height clamp, or a resize mid-run). Phones run
portrait only. Input maps through the camera (`Camera::viewport_to_world_2d`) and is
clamped to the field, so a finger in the letterbox stays on the nearest wall. Walls,
spawn areas, the area box and the HUD derive from the `Field`, never from the window
([019]). The title and game over screens are world-space too, centered on the origin.

**Physics.** avian2d with default plugins and `enhanced-determinism` (libm math, so the
results do not depend on the CPU; no measurable cost on desktop, [072]). Walls and
claimed shapes are static bodies.
Enemies and items are dynamic bodies with zero gravity, except `Square`. Line segments
are colliders with no body. Layers: the player layer (line) meets enemies, items and
bullets; enemies meet enemies, the line and the default layer (walls, shapes); items meet
the line and the default layer; Shooter bullets (sensors) meet the line and the default
layer only. A time stop locks the axes of enemies and bullets ([046]). `GameLayer::Shape` is not used. Collision checks read
`CollisionStart` messages; every `Collider` sends them (`CollisionEventsEnabled`).

**Rendering.** One HDR camera: bloom, TonyMcMapface, SMAA, then the CRT pass. Claimed
shapes, the menu box and the arrow are `Shape` meshes; line segments are sprites; walls,
balls and squares are `Mesh2d` with a `ColorMaterial`; sparks are sprites; effects are
`Material2d`. Text is `Text2d` everywhere (title, HUD, pause, game over, item countdown),
so it gets the bloom and the CRT pass.

**Logging.** `info!` lines are for desktop and debug builds: `Cargo.toml` turns on
`release_max_level_warn` for `log` and `tracing`, so release builds (also mobile release
builds) keep only `warn!` and `error!` ([026]).

**Randomness.** Gameplay randomness uses `GameRng`, a `fastrand::Rng` seeded from
`Run.seed`. Each new level reseeds it with `seed + level`. Only the seed of a new
run comes from the global `fastrand` (or from a typed code, [067]). Enemies spawn before items on level start, so a
seed gives one spawn order. The area measure uses no RNG ([024]). A seed replays a shot
run, also with an arrow ([026]), and gives the same state hash at 30, 60 and 120 frames
per second ([072], `just replay`), on x86_64 Linux and arm64 macOS (the CI `replay` job,
against `tools/replay-hashes.txt`, [075]). A `usize` draw (`usize`, `choice`, `shuffle`)
differs on 32-bit wasm, so gameplay draws use `u64` (`pick_kind`, the bomb's victim).
Float functions go through `bevy::math::ops`: avian's `enhanced-determinism` turns on
`bevy_math/libm`, so they use one portable libm, not the platform's (`clippy.toml`
forbids the `f32` methods). Apple's `sin`, `cos` and `atan2` gave seed 8 another state
than glibc's ([075]).

**Dev tools.** With `RIX_SHOT`, the `base` plugin leaves out winit and the `dev_tools`
plugin adds the schedule runner and a fixed time step per frame: 1/60 s, or
`1/RIX_SHOT_FPS`. The primary `Window` entity still exists (no OS window behind it). The
camera draws into an image of the window size or `RIX_SHOT_SIZE` (`render_offscreen`).
`drive_line` (`FixedPreUpdate`) sends the scripted touch input, one touch per game tick
(a line in world units, mapped through the camera, sent as `TouchInput` and as
`WindowEvent`, as winit does; bevy_picking holds a pointer entity for the touch), and `log_state` logs a hash of the
game state (`RIX_STATE`) after 160 game ticks ([072]). `drive_shot` takes a `Screenshot`
of the image, which ends the run. The replay scene has no renderer (`WgpuSettings`
with no backends, so no GPU: CI runs it) and no picture: `log_state` ends the run. The
script counts from the first frame of the title screen, and game time
stands still while the assets load, so the loading time does not change the picture
([027]). The flow scene (`drive_flow`) also clears the level, sends `Die` and taps
RETRY (`game_over::RETRY_Y`); `log_census` logs the entity counts (without resource entities) when they
change. Every `dev_tools` build times the main schedule (`First` to `Last`: game logic and
physics, not rendering) and warns `slow update` when it takes over 16 ms ([024]).

## Invariants

- Entity lifetimes:
  - `DespawnOnExit(Phase::Playing)` (enemies, items, line segments, item countdown,
    HUD): gone at a level transition and at game over.
  - `DespawnOnExit(Screen::Gameplay)`: `Wall` (stays between levels) and `DrawnShape`.
    Between levels a `DrawnShape` fades out and despawns itself; the next level starts
    only when none is left.
  - The title, settings, seed and game over screens: `DespawnOnExit(Screen::Title)`,
    `DespawnOnExit(Screen::Settings)`, `DespawnOnExit(Screen::Seed)` and
    `DespawnOnExit(Screen::GameOver)`; the pause overlay: `DespawnOnExit(IsPaused::Paused)`.
  - Explosions: only `DespawnAfter` (2 s) removes them.
  - `UserPath` and `InputState` reset on `OnExit(Phase::Playing)`.
- One drawn path at a time, from one pointer: a second touch is ignored while one is down.
- A path starts and ends on the `FreeRegion` boundary, touches it nowhere else, and does
  not cross itself. `FreeRegion` stays one simple counter-clockwise polygon.
- `FreeRegion` does not repeat the first point at the end.
- Gameplay never calls the global `fastrand` functions (AGENTS.md hard rule).
- Shapes are own lyon meshes, never `bevy_prototype_lyon` (D2).
