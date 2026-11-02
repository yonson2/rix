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

One `App` holds everything. `rix::app()` adds `BasePlugin` (Bevy, physics, camera, CRT)
and `RixPlugin` (all game logic), and `DevToolsPlugin` with the `dev_tools` feature. The
desktop binary and the mobile library (`mobile/`) both run it.

## Code map

### Crate root (`src/`)

- `main.rs`: the desktop binary; runs `rix::app()`.
- `lib.rs`: the module list and `app()`. The binary, the mobile crate, the examples and the
  tests use the library.
- `base`: `BasePlugin`. `DefaultPlugins` with one 625 x 625 window that does not resize,
  avian `PhysicsPlugins`, `PhysicsPickingPlugin`, `CameraPlugin`, `CrtPlugin`. It makes
  every `Collider` require `CollisionEventsEnabled`. In a shot run it leaves out
  `WinitPlugin` ([064]).
- `camera`: `CameraPlugin` spawns the one `Camera2d` in `PreStartup` with `Hdr`,
  `Tonemapping::TonyMcMapface`, `Bloom`, `Smaa` (no MSAA) and `PostProcessSettings`.
- `crt`: `CrtPlugin`, the CRT post-process. `PostProcessSettings` is extracted to the
  render world. The render system `post_process` runs in `Core2dSystems::PostProcess`,
  after tonemapping and SMAA, so it is the last effect. Shader: `crt_shader.wesl`.
- `shape`: `Shape` (a lyon `Path`, an optional fill and an optional `Stroke`) and
  `ShapePlugin`. When a `Shape` changes, `mesh_shapes` tessellates it into one mesh with
  vertex colors, drawn with one shared white `ColorMaterial` (`ShapeMaterial`). Helpers:
  `polygon`, `rounded_polygon`, `rounded_rectangle`. This replaces bevy_prototype_lyon
  (D2).
- `colors`: color constants. `NEON_*` values are above 1.0, so they bloom in HDR.
- `common`: `Transition`, a timer component (menu title fade, enemy spawn animation).
- `utils`: `ColorTransition` and `update_sprite_color` / `update_mesh_color` (color
  pulses), `FadeShape` and `update_shape_fade` (claimed shapes fade out between levels),
  `generate_enemy_position` (a free spot in the field, from the level RNG).
- `menu`: `MenuPlugin`. The title screen in `AppState::Menu`; a tap or click goes to
  `InGame`.
- `dev_tools` (feature `dev_tools` only): `DevToolsPlugin`. `RIX_SEED` sets the seed of
  the first run. `RIX_SHOT` runs a scripted game (or the menu with `RIX_SHOT_SCENE=menu`)
  without a window, saves one picture and exits. See "Dev tools" below.

### Game (`src/rix/`)

- `mod.rs`: `RixPlugin` adds the states and the plugins below. `RixComponent` marks what
  dies on `OnExit(AppState::InGame)` (`despawn_screen`). `GameLayer` and
  `get_player_layer` / `get_enemy_layer` / `get_item_layer` set the collision layers.
- `state`: `AppState` (`Setup`, `Menu`, `InGame`, `GameOver`, `LevelTransition`) and two
  sub-states: `IsPaused` (in `InGame`) and `TransitionState` (in `LevelTransition`).
- `setup`: `SetupPlugin` adds `EventsPlugin` and `EffectsPlugin`, registers the spawn
  systems in `OnEnter(Setup)` and goes to `Menu`. `LevelInfo` is the run state: level,
  target (0.75), claimed fraction (`current`), `Points`, kill count, seed and the seeded
  `rng`. `handle_current_level` starts the level transition when the target is reached.
- `events`: the messages between systems (`DrawLine`, `ClearLines`, `BuildPolygon`,
  `DrawPolygon`, `Die`, `ActivateItem`, `ActivateBasicBomb`, `IncreasePoints`,
  `UpdatePoints`, `InputPosition`), the `DespawnEnemy` event (an observer trigger) and
  `ShapeMeta` (the wall and point where a claimed shape starts).
- `user_input`: `UserInputPlugin`. `handle_input_events` turns mouse and touch into
  `InputState` (one active pointer, world coordinates). `handle_input` keeps the line
  being drawn in `UserPath` and sends the line messages. Pure helpers with tests:
  `convert_to_world_coords`, `get_edge_point`, `would_create_intersection`.
- `wall`: `WallPlugin`. Four `Wall` entities (`WallSide`) with static colliders, spawned
  on `OnEnter(InGame)` when none exist. `WALL_THICKNESS` (35) and `WALL_THICKNESS_TOP`
  (65, room for the HUD). `get_world_inner_corners` gives the `WallCorners` of the field.
  `WallEvent` makes the walls glow while a line is drawn.
- `polygon`: `PolygonPlugin`. The line segments (`TempLineSegment`), the claim
  (`PolygonData` -> `get_possible_polygons` -> `get_correct_polygon`) and the claimed
  shapes (`DrawnShape` with `Territory`, the outline). `is_point_in_polygon` is the one
  point test the game uses.
- `level`: `LevelPlugin`. `calculate_area_progress` (claimed fraction), points
  (`ENEMY_POINTS`, `ITEM_POINTS`, doubled per level), the level transition, and
  `DespawnAfter` (a timer that despawns its entity).
- `enemies`: `EnemiesPlugin`, `Enemy`, `EnemySystems` (one-shot spawn systems),
  `spawn_enemies`, death on line contact, and the two enemy-removal observers.
  - `ball`: `Ball`, moves at a fixed speed, gets a random turn on each hit.
  - `square`: `Square`, falls up or down (`GravityScale`), pulses and teleports.
  - `arrow`: `Arrow`, turns toward the finger (`InputPosition`) and glows while a line is
    drawn.
  - `rng`: `calculate_enemy_count` (per level, pure, tested).
- `items`: `ItemsPlugin`, `Item`, `ItemSystems`. Items spawn on level start (one coin
  flip), blink and show a countdown before they despawn, and activate when a line touches
  them or a claim covers them.
  - `basic_bomb`: the only item. It removes one random enemy, with an explosion.
- `effects`: `EffectsPlugin` and the `Material2d` types: `ExplosionMaterial` (bomb),
  `PixelExplosionMaterial` (removed enemy), `GlowMaterial` (the bomb item).
- `ui`: `UiPlugin`, the in-game HUD (level, points, progress) as Bevy UI nodes.
- `game_over`: `GameOverPlugin`, the game over screen; a tap starts a new run.

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
  `just android` puts the library into its `jniLibs` ([021]).
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
Setup --> Menu --tap--> InGame --line touched--> GameOver --tap--> InGame (new run)
                          |  ^
           75 % claimed   v  |  no claimed shape left
                     LevelTransition (Setup -> Loading)
```

- `IsPaused` exists only in `InGame` and is always `Running`: nothing enters `Paused` yet
  ([034]). Most gameplay systems run with `run_if(in_state(IsPaused::Running))`, so they
  stop in every other state.
- `LevelTransition`: `start_transition` gives each `DrawnShape` a `FadeShape` and goes to
  `Loading`. When the last shape is gone, `handle_transition_loading` raises the level,
  reseeds the RNG and goes back to `InGame`.
- `GameOver` -> `InGame` calls `LevelInfo::reset`: a new run with a new seed.

## Main data flow: one line

1. `handle_input_events` reads `MouseButtonInput`, `TouchInput` and `CursorMoved`, and
   updates `InputState`.
2. `handle_input`: a press on a wall (or inside a `Territory`) starts a path in
   `UserPath` (not on a wall part that is already claimed) and sends `WallEvent::Bloom`.
   Each move in open space sends `DrawLine`. A move that crosses the path cancels it
   (`ClearLines`, `WallEvent::Reset`). A release cancels it too. While drawing it sends
   `InputPosition` (the arrow follows it).
3. `handle_draw_line` spawns one `TempLineSegment` per `DrawLine`: a sprite with a
   rectangle collider on the player layer, with `RixComponent`.
4. When the path reaches a wall or a `Territory`, `handle_input` sends `ClearLines` and
   `BuildPolygon(PolygonData)`.
5. `handle_build_polygon`: if the path starts or ends inside a claimed shape, it adds that
   shape's outline up to its `ShapeMeta` point, so the path reaches a wall.
   `get_possible_polygons` closes the path both ways round the inner corners.
   `get_correct_polygon` takes the polygon with no enemy inside. If both have enemies, it
   takes the first one, and the enemies inside it are eaten (`DespawnEnemy`). Items inside
   the polygon get `ActivateItem`. Then it sends `DrawPolygon`.
6. `handle_draw_polygon` spawns the claimed shape: `DrawnShape`, `Territory`, a filled
   `Shape`, `ShapeMeta`, `RigidBody::Static` and a convex-decomposition collider.
7. `calculate_area_progress` runs when a `Territory` is added. It samples 100,000 random
   points and sets `LevelInfo.current`. A gain above 1 % sends `UpdatePoints`.
   `handle_update_points` adds points (times `2^(level - 1)`).
8. `handle_current_level` sees `current >= target` and goes to `LevelTransition`.

Death: `handle_enemy_collision` reads avian `CollisionStart`. An `Enemy` with a
`TempLineSegment` sends `Die`, and `handle_die` goes to `GameOver`.

Enemy removal: `on_despawn_enemy` (observer of `DespawnEnemy`) despawns the enemy, counts
a kill and sends `IncreasePoints`. `on_remove_enemy` (observer of `Remove<Enemy>`) spawns
a pixel explosion and counts a kill again. It also runs when `OnExit(InGame)` despawns
the enemies ([030]).

## Cross-cutting concerns

**Coordinates.** World origin is the window center, y up, one world unit per logical
pixel. The window is 625 x 625 logical and the camera does not move or zoom, so input
maps to the world with `convert_to_world_coords` (window math, not the camera). Walls,
spawn areas, the area sampler and the HUD all derive from the window size ([019]).

**Physics.** avian2d with default plugins. Walls and claimed shapes are static bodies.
Enemies and items are dynamic bodies with zero gravity, except `Square`. Line segments
are colliders with no body. Layers: the player layer (line) meets enemies and items;
enemies meet enemies, the line and the default layer (walls, shapes); items meet the line
and the default layer. `GameLayer::Shape` is not used. Collision checks read
`CollisionStart` messages; every `Collider` sends them (`CollisionEventsEnabled`).

**Rendering.** One HDR camera: bloom, TonyMcMapface, SMAA, then the CRT pass. Claimed
shapes, the menu box and the arrow are `Shape` meshes; line segments are sprites; walls,
balls and squares are `Mesh2d` with a `ColorMaterial`; effects are `Material2d`. Text: Bevy UI nodes for the
HUD and the game over screen, `Text2d` for the menu and the item countdown.

**Randomness.** Gameplay randomness uses `LevelInfo.rng`, a `fastrand::Rng` seeded from
`LevelInfo.seed`. Each new level reseeds it with `seed + level`. Only the seed of a new
run comes from the global `fastrand`. Enemies spawn before items on level start, so a
seed gives one spawn order. The area sampler also draws from this RNG ([028]). A seed
replays a run, except with an arrow ([026]).

**Dev tools.** With `RIX_SHOT`, `BasePlugin` leaves out winit and `DevToolsPlugin` adds
the schedule runner at 1/60 s and a fixed 1/60 s time step. The primary `Window` entity
still exists (no OS window behind it), so the game reads the same size. The camera draws
into a 625 x 625 image (`render_offscreen`), `drive_shot` sends scripted touch input, and
a `Screenshot` of the image ends the run.

## Invariants

- Entity lifetimes:
  - `RixComponent` (enemies, items, line segments, item countdown, HUD): despawned on
    `OnExit(AppState::InGame)`, so also between levels.
  - `DrawnShape`: no `RixComponent`. Between levels it fades out and despawns itself; the
    next level starts only when none is left. `OnEnter(GameOver)` and `OnEnter(Menu)`
    despawn it.
  - `Wall`: spawned on `OnEnter(InGame)` only when no wall exists, so walls stay between
    levels. `OnEnter(GameOver)` and `OnEnter(Menu)` despawn them.
  - Explosions: only `DespawnAfter` (2 s) removes them.
  - `UserPath` and `InputState` reset on `OnExit(InGame)`.
- One drawn path at a time, from one pointer: a second touch is ignored while one is down.
- A path starts and ends on a wall or inside a claimed shape, and does not cross itself.
- `Territory` outlines are closed: the last point repeats the first.
- Gameplay never calls the global `fastrand` functions (AGENTS.md hard rule).
- Shapes are own lyon meshes, never `bevy_prototype_lyon` (D2).
