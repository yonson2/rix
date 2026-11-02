# Architecture

This file describes Rix as it IS. The target shape lives in the issues ([026] and M3).
Update this file in the same commit as a structural change (modules, plugins, states,
events, dependencies). Names are given without links, so search for them (`grep`, or
symbol search in the editor).

## Bird's-eye view

Rix is a Qix-like game on Bevy 0.20 with avian2d 0.8 physics. The player presses on a
wall and drags a line across the field. Each part of the line is a short-lived entity with
a collider. If an enemy touches the line, the player dies. If the line reaches a wall (or
an area that is already claimed), the line and the walls close two polygons. The polygon
without enemies becomes claimed area: a filled shape with a static collider. When the
claimed part of the field reaches 75 %, the level ends, the shapes fade out and the next
level starts with more enemies.

One `App` holds everything. `rix::app()` adds the plugins of `base` (Bevy, physics,
camera, CRT), `screens` (the `Screen` state, title and game over) and `game` (all game
logic), and `dev_tools` with the `dev_tools` feature. The desktop binary and the mobile
library (`mobile/`) both run it.

Each module has its own plugin, `pub(super) fn plugin(app: &mut App)`, added by its
parent module ([026], the bevy_new_2d pattern). Below, "plugin" means that function.

## Code map

### Crate root (`src/`)

- `main.rs`: the desktop binary; runs `rix::app()`.
- `lib.rs`: the module list and `app()`. The binary, the mobile crate, the examples and the
  tests use the library.
- `base`: plugin. `DefaultPlugins` with one resizable 450 x 800 window (the field),
  avian `PhysicsPlugins`, `PhysicsPickingPlugin`, the camera and CRT plugins. It makes
  every `Collider` require `CollisionEventsEnabled`. In a shot run it leaves out
  `WinitPlugin` ([064]).
- `camera`: the plugin spawns the one `Camera2d` in `PreStartup` with `Hdr`,
  `Tonemapping::TonyMcMapface`, `Bloom`, `Smaa` (no MSAA) and `PostProcessSettings`.
- `crt`: the CRT post-process. `PostProcessSettings` is extracted to the
  render world. The render system `post_process` runs in `Core2dSystems::PostProcess`,
  after tonemapping and SMAA, so it is the last effect. Shader: `crt_shader.wesl`.
- `shape`: `Shape` (a lyon `Path`, an optional fill and an optional `Stroke`) and its
  plugin. When a `Shape` changes, `mesh_shapes` tessellates it into one mesh with
  vertex colors, drawn with one shared white `ColorMaterial` (`ShapeMaterial`). Helpers:
  `polygon`, `rounded_polygon`, `rounded_rectangle`. This replaces bevy_prototype_lyon
  (D2).
- `asset_tracking`: `LoadResource` (`app.load_resource::<T>()`) and `ResourceHandles`,
  from bevy_new_2d ([027]). A collection is a `Resource + Asset` struct of `#[dependency]`
  handles built in `FromWorld`; it becomes a resource once every handle has loaded.
- `theme`: the shared look. `FontAssets` (the font, a `LoadResource` collection).
  `theme::palette`: color constants. `NEON_*` values are above 1.0, so they bloom in HDR.
  UI widgets join it with [036].
- `common`: `Transition`, a timer component (menu title fade, enemy spawn animation).
- `utils`: `ColorTransition` and `update_sprite_color` / `update_mesh_color` (color
  pulses), `FadeShape` and `update_shape_fade` (claimed shapes fade out between levels),
  `generate_enemy_position` (a free spot in the field, from the level RNG).
- `screens`: the `Screen` state (`Loading`, `Title`, `Gameplay`, `GameOver`, [026]).
  - `loading`: the first screen (blank); it goes to `Title` when
    `ResourceHandles::is_all_done` ([027]).
  - `title`: the title screen in `Screen::Title`; a tap or click goes to
    `Screen::Gameplay`.
  - `game_over`: the game over screen; a tap starts a new run.
- `dev_tools` (feature `dev_tools` only): plugin. `RIX_SEED` sets the seed of
  the first run. `RIX_SHOT` runs a scripted game (or the menu with `RIX_SHOT_SCENE=menu`,
  or a level clear, death and retry with `RIX_SHOT_SCENE=flow`) without a window, saves
  one picture and exits. See "Dev tools" below.

### Game (`src/game/`)

- `mod.rs`: the plugin adds the sub-states, the system sets (`AppSystems`,
  `PausableSystems`) and the plugins below. `GameLayer` and
  `get_player_layer` / `get_enemy_layer` / `get_item_layer` set the collision layers.
- `state`: the sub-states of `Screen::Gameplay`: `Phase` (`Playing`, `LevelTransition`),
  then `IsPaused` in `Phase::Playing` and `TransitionState` in `Phase::LevelTransition`
  ([026]). Entities carry `DespawnOnExit` of the state they belong to (see Invariants).
- `setup`: the plugin adds the `events` and `effects` plugins, registers the spawn
  systems in `OnEnter(Screen::Loading)`. Three resources hold the game state ([028]):
  `Run` (seed, level, `Points`, kill count), `LevelProgress` (target 0.75, claimed
  fraction `current`) and `GameRng` (the seeded `fastrand::Rng`). `new_run` inserts all
  three for a seed. `handle_current_level` starts the level transition when the target is
  reached.
- `events`: the messages between systems (`DrawLine`, `ClearLines`, `BuildPolygon`,
  `DrawPolygon`, `Die`, `ActivateItem`, `ActivateBasicBomb`, `IncreasePoints`,
  `UpdatePoints`, `InputPosition`), the `DespawnEnemy` event (an observer trigger) and
  `ShapeMeta` (the wall and point where a claimed shape starts).
- `user_input`: `handle_input_events` turns mouse and touch into
  `InputState` (one active pointer, world coordinates). `handle_input` keeps the line
  being drawn in `UserPath` and sends the line messages. Pure helpers with tests:
  `get_edge_point`, `would_create_intersection`.
- `wall`: four `Wall` entities (`WallSide`) with static colliders, spawned
  on `OnEnter(Screen::Gameplay)`. `FIELD` (450 x 800, walls included, D17) is the
  playfield; `WALL_THICKNESS` (35) and `WALL_THICKNESS_TOP` (65, room for the HUD).
  `get_world_inner_corners` gives the `WallCorners` of the field.
  `WallEvent` makes the walls glow while a line is drawn.
- `polygon`: the line segments (`TempLineSegment`), the claim
  (`PolygonData` -> `get_possible_polygons` -> `get_correct_polygon`) and the claimed
  shapes (`DrawnShape` with `Territory`, the outline). `is_point_in_polygon` is the one
  point test the game uses.
- `level`: `calculate_area_progress` (claimed fraction), points
  (`ENEMY_POINTS`, `ITEM_POINTS`, doubled per level), the level transition, and
  `DespawnAfter` (a timer that despawns its entity).
- `enemies`: `Enemy`, `EnemySystems` (one-shot spawn systems),
  `spawn_enemies`, death on line contact, and the two enemy-removal observers.
  - `ball`: `Ball`, moves at a fixed speed, gets a random turn on each hit.
  - `square`: `Square`, falls up or down (`GravityScale`), pulses and teleports.
  - `arrow`: `Arrow`, turns toward the finger (`InputPosition`) and glows while a line is
    drawn.
  - `rng`: `calculate_enemy_count` (per level, pure, tested).
- `items`: `Item`, `ItemSystems`. Items spawn on level start (one coin
  flip), blink and show a countdown before they despawn, and activate when a line touches
  them or a claim covers them.
  - `basic_bomb`: the only item. It removes one random enemy, with an explosion.
    `ItemAssets` holds its image.
- `effects`: the `Material2d` types: `ExplosionMaterial` (bomb),
  `PixelExplosionMaterial` (removed enemy), `GlowMaterial` (the bomb item).
- `ui`: the in-game HUD (level, points, progress) as `Text2d` in the world,
  under the top wall. It redraws only when `Run` or `LevelProgress` changes ([028]).
- `pause`: the pause triggers and the "PAUSED" overlay (world `Text2d` over
  a dim sprite, despawned on exit with `DespawnOnExit`).

### Shaders (`assets/shaders/`, WESL since [018])

- `crt_shader.wesl`: the CRT pass (`crt`).
- `cartoon_explosion.wesl`: `ExplosionMaterial`.
- `pixel_explosion.wesl`: `PixelExplosionMaterial`.
- `glow_material.wesl`: `GlowMaterial`.
- `flame_material.wesl`: only the `flame_shader` example uses it.

### Outside `src/`

- `mobile/`: the `rix_mobile` crate (a workspace member): a `cdylib` whose `#[bevy_main]`
  runs `rix::app()`, with Bevy's `android-game-activity` feature. `mobile/android/` is the
  Gradle project (GameActivity, `top.cosas.rix`, version from the root `Cargo.toml`);
  `just android` puts the library into its `jniLibs` ([021]). `MainActivity` hides the
  system bars and turns Back into "move to the background", as Home does ([023]).
- `mobile/ios/`: the Xcode project (`Rix.xcodeproj`, `top.cosas.rix`, iOS 15.0). Its "Rust"
  build phase (`build_rust_deps.sh`) builds the root `rix` binary for the iOS target, and
  the app bundles the root `assets/` folder. `just ios-sim` passes the version from
  `Cargo.toml` ([020]).
- `examples/`: one example per shader (`glow_material`, `cartoon_explosion`,
  `pixel_explosion`, `flame_shader`).
- `assets/`: the font (`fonts/main.ttf`), the bomb image and the shaders. Licences: [011].
- `issues/`: the tracker. `tracker/`: the Astro issue board that reads it ([004]).
- `.github/workflows/ci.yml`: CI (fmt, clippy, tests) on pushes and PRs to `main` ([008]).
- `docs/research/`: studies that issues link to. `docs/design/`: feature specs.

## State machine

```
Screen:  Loading --assets loaded--> Title --tap--> Gameplay --line touched--> GameOver --tap--> Gameplay (new run)
                                                      |
Phase (in Gameplay):  Playing --75 % claimed--> LevelTransition (Setup -> Loading)
                         ^                              |
                         +---- no claimed shape left ---+
```

- `IsPaused` exists only in `Phase::Playing`. `pause` enters `Paused` when the app suspends, the
  window loses focus or Escape is pressed, and stops `Time<Virtual>` and `Time<Physics>`;
  the release of a tap or click (or Escape) goes back to `Running`. Both switches drop the
  pending pointer messages, and leaving `Paused` cancels a line in progress ([034]). The
  gameplay systems are in the `PausableSystems` set, which runs only in
  `IsPaused::Running`, so they stop in every other state.
- System order in `Update` ([026]): the sets `AppSystems::TickTimers`, `RecordInput` and
  `Update` run in that order (`src/game/mod.rs`). `user_input` records input; the systems
  that read its messages (line, claim, walls, arrow) are in `AppSystems::Update`, so they
  see the input of the same frame. Systems inside one set have no order unless they are
  chained ([072]).
- `Phase::LevelTransition`: `start_transition` gives each `DrawnShape` a `FadeShape` and
  goes to `Loading`. When the last shape is gone, `handle_transition_loading` raises the
  level, reseeds the RNG and goes back to `Phase::Playing`.
- `GameOver` -> `Gameplay` calls `new_run` with a random seed.

## Main data flow: one line

1. `handle_input_events` reads `MouseButtonInput`, `TouchInput` and `CursorMoved`, and
   updates `InputState`.
2. `handle_input`: a press on a wall (or inside a `Territory`) starts a path in
   `UserPath` (not on a wall part that is already claimed) and sends `WallEvent::Bloom`.
   Each move in open space sends `DrawLine`. A move that crosses the path cancels it
   (`ClearLines`, `WallEvent::Reset`). A release cancels it too. While drawing it sends
   `InputPosition` (the arrow follows it).
3. `handle_draw_line` spawns one `TempLineSegment` per `DrawLine`: a sprite with a
   rectangle collider on the player layer, with `DespawnOnExit(Phase::Playing)`.
4. When the path reaches a wall or a `Territory`, `handle_input` sends `ClearLines` and
   `BuildPolygon(PolygonData)`.
5. `handle_build_polygon`: if the path starts or ends inside a claimed shape, it adds that
   shape's outline up to its `ShapeMeta` point, so the path reaches a wall.
   `get_possible_polygons` closes the path both ways round the inner corners.
   `get_correct_polygon` takes the polygon with no enemy inside. If both have enemies, it
   takes the first one, and the enemies inside it are eaten (`DespawnEnemy`). Items inside
   the polygon get `ActivateItem`. Then it sends `DrawPolygon`.
6. `handle_draw_polygon` spawns the claimed shape: `DrawnShape`, `Territory`, a filled
   `Shape`, `ShapeMeta`, `RigidBody::Static` and a trimesh collider of the shape's fill
   triangles (`territory_collider`, the same lyon fill as the mesh, [024]).
7. `calculate_area_progress` runs when a `Territory` is added. `claimed_fraction` measures
   the union of all `Territory` outlines on one scanline per world unit, and it sets
   `LevelProgress.current` ([024]; the box is off by the top wall, [029]). A gain above 1 %
   sends `UpdatePoints`. `handle_update_points` adds points (times `2^(level - 1)`).
8. `handle_current_level` sees `current >= target` and goes to `Phase::LevelTransition`.

Death: `handle_enemy_collision` reads avian `CollisionStart`. An `Enemy` with a
`TempLineSegment` sends `Die`, and `handle_die` goes to `Screen::GameOver`.

Enemy removal: `on_despawn_enemy` (observer of `DespawnEnemy`) despawns the enemy, counts
a kill and sends `IncreasePoints`. `on_remove_enemy` (observer of `Remove<Enemy>`) spawns
a pixel explosion and counts a kill again. It also runs when the exit from
`Phase::Playing` despawns the enemies ([030]).

## Cross-cutting concerns

**Coordinates.** World origin is the field center, y up. The field is a fixed 450 x 800
world units (`wall::FIELD`, D17) on every device. The camera's `ScalingMode::AutoMin` fits
the whole field into any window or image and letterboxes the rest, so a resize or a
rotation changes only the view. Input maps through the camera
(`Camera::viewport_to_world_2d`) and is clamped to the field, so a finger in the
letterbox stays on the nearest wall. Walls, spawn areas, the area box and the HUD
derive from `FIELD`, never from the window ([019]). The game over screen is UI nodes in
window pixels.

**Physics.** avian2d with default plugins. Walls and claimed shapes are static bodies.
Enemies and items are dynamic bodies with zero gravity, except `Square`. Line segments
are colliders with no body. Layers: the player layer (line) meets enemies and items;
enemies meet enemies, the line and the default layer (walls, shapes); items meet the line
and the default layer. `GameLayer::Shape` is not used. Collision checks read
`CollisionStart` messages; every `Collider` sends them (`CollisionEventsEnabled`).

**Rendering.** One HDR camera: bloom, TonyMcMapface, SMAA, then the CRT pass. Claimed
shapes, the menu box and the arrow are `Shape` meshes; line segments are sprites; walls,
balls and squares are `Mesh2d` with a `ColorMaterial`; effects are `Material2d`. Text: Bevy UI nodes for the
game over screen, `Text2d` for the menu, the HUD and the item countdown.

**Logging.** `info!` lines are for desktop and debug builds: `Cargo.toml` turns on
`release_max_level_warn` for `log` and `tracing`, so release builds (also mobile release
builds) keep only `warn!` and `error!` ([026]).

**Randomness.** Gameplay randomness uses `GameRng`, a `fastrand::Rng` seeded from
`Run.seed`. Each new level reseeds it with `seed + level`. Only the seed of a new
run comes from the global `fastrand`. Enemies spawn before items on level start, so a
seed gives one spawn order. The area measure uses no RNG ([024]). Since the
`AppSystems` order ([026]), a seed replays a shot run, also with an arrow (seeds 3, 7 and
10 give one picture in 6-10 runs each); other frame rates and devices are [072].

**Dev tools.** With `RIX_SHOT`, the `base` plugin leaves out winit and the `dev_tools`
plugin adds the schedule runner at 1/60 s and a fixed 1/60 s time step. The primary
`Window` entity still exists (no OS window behind it). The camera draws into an image of
the window size or `RIX_SHOT_SIZE` (`render_offscreen`), `drive_shot` sends scripted touch
input (a line in world units, mapped through the camera), and a `Screenshot` of the image
ends the run. The script counts from the first frame of the title screen, and game time
stands still while the assets load, so the loading time does not change the picture
([027]). The flow scene (`drive_flow`) also clears the level, sends `Die` and taps to
play again; `log_census` logs the entity counts (without resource entities) when they
change. Every `dev_tools` build times the main schedule (`First` to `Last`: game logic and
physics, not rendering) and warns `slow update` when it takes over 16 ms ([024]).

## Invariants

- Entity lifetimes:
  - `DespawnOnExit(Phase::Playing)` (enemies, items, line segments, item countdown,
    HUD): gone at a level transition and at game over.
  - `DespawnOnExit(Screen::Gameplay)`: `Wall` (stays between levels) and `DrawnShape`.
    Between levels a `DrawnShape` fades out and despawns itself; the next level starts
    only when none is left.
  - The title and game over screens: `DespawnOnExit(Screen::Title)` and
    `DespawnOnExit(Screen::GameOver)`; the pause overlay: `DespawnOnExit(IsPaused::Paused)`.
  - Explosions: only `DespawnAfter` (2 s) removes them.
  - `UserPath` and `InputState` reset on `OnExit(Phase::Playing)`.
- One drawn path at a time, from one pointer: a second touch is ignored while one is down.
- A path starts and ends on a wall or inside a claimed shape, and does not cross itself.
- `Territory` outlines are closed: the last point repeats the first.
- Gameplay never calls the global `fastrand` functions (AGENTS.md hard rule).
- Shapes are own lyon meshes, never `bevy_prototype_lyon` (D2).
