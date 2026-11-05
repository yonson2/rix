//! Dev-only tools, compiled with the `dev_tools` feature ([006]). Release builds never
//! contain them.
//!
//! - `RIX_SEED=<u64>`: the seed of the first run.
//! - `RIX_LEVEL=<n>`: with `RIX_SEED`, the first run starts at level n, with the enemies,
//!   items and seed that level has after a clear (store screenshots, [051]).
//! - `RIX_CLOCK=<secs>`: with `RIX_SEED`, the first run has a countdown of that many
//!   seconds ([122]): a shot of the last seconds or of "TIME!".
//! - `RIX_SHOT=<file.png>`: skip the menu, draw one line across the bottom of the field
//!   with touch input, save a screenshot to the file and exit. Each frame advances time by
//!   a fixed step, so the run does not depend on the frame rate ([062]).
//! - `RIX_SHOT_SCENE=menu`: with `RIX_SHOT`, screenshot the menu instead.
//! - `RIX_SHOT_SCENE=flow`: with `RIX_SHOT`, the game line, then a level clear, a death
//!   and a retry; logs an entity census (`RIX_FLOW`) each time it changes ([026]).
//! - `RIX_SHOT_SCENE=store`: with `RIX_SHOT`, three lines that claim two parts of the field;
//!   the shot shows the third line half drawn (store screenshots, [051]).
//! - `RIX_SHOT_LINES=<n>`: with the store scene, its line set (default 0, [103]).
//! - `RIX_SHOT_SCENE=kill`: with `RIX_SHOT`, every enemy dies at once before the game line
//!   (so a high `RIX_LEVEL` cannot die first); the shot shows the kill effects ([116]).
//! - `RIX_SHOT_SCENE=replay`: with `RIX_SHOT`, the game line without the renderer (no GPU,
//!   so CI runs it, [075]): it logs `RIX_STATE`, exits and saves no picture.
//! - `RIX_SHOT_SIZE=<w>x<h>`: with `RIX_SHOT`, the image size (default: the window size).
//!   The field letterboxes into any size ([019]).
//! - `RIX_SHOT_FPS=<n>`: with `RIX_SHOT`, frames per second (default 60). The line script
//!   counts game ticks (`FixedUpdate`), and the run logs a state hash (`RIX_STATE`) after
//!   `STATE_TICK` ticks: a seed gives the same hash at any frame rate ([072]).
//! - Always: a warning for each update over 16 ms ([024]).
//!
//! A `RIX_SHOT` run opens no window and does not take the focus: `BasePlugin` leaves out
//! winit, the schedule runner drives the frames and the camera draws into an image ([064]).

use std::{
    f32::consts::PI,
    path::PathBuf,
    time::{Duration, Instant},
};

use avian2d::prelude::{LinearVelocity, Position};
use bevy::{
    app::ScheduleRunnerPlugin,
    camera::{ImageRenderTarget, RenderTarget},
    ecs::resource::IsResource,
    input::touch::TouchPhase,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    },
    time::TimeUpdateStrategy,
    window::{PrimaryWindow, WindowEvent},
};

use crate::game::{
    countdown::Countdown,
    enemies::Enemy,
    events::{DespawnEnemy, Die},
    items::Item,
    polygon::{DrawnShape, TempLineSegment, Territory},
    setup::{GameRng, LevelProgress, Run, by_place, new_run},
    skins::{FillSkin, KillSkin, LineSkin, Skins, WallSkin},
    state::Phase,
    territory::FreeRegion,
    wall::{Field, Wall},
};
use crate::screens::{Screen, game_over};

/// The scripted line: from inside the left wall to inside the right wall, a little above
/// the bottom wall, in world units from the field's bottom corners.
const LINE_INSET: Vec2 = Vec2::new(12.5, 87.5);
/// Game ticks (`FixedUpdate`, 64 per second) since the game started.
const LINE_TICK: u32 = 64;
const LINE_TICKS: u32 = 12;
const STATE_TICK: u32 = 160;
/// The store scene's line sets ([051], [103]); `RIX_SHOT_LINES` picks one. A line is
/// (start, end, half waves, sway): from wall to wall in fractions of the half field, it
/// sways off the straight path like a finger (`sway`, in fractions of the half width).
/// Each line starts `STORE_EVERY` ticks after the last and takes `STORE_LINE_TICKS`; the
/// shot comes when the last line is at `STORE_SHOT_STEP`.
const STORE_LINES: [[(Vec2, Vec2, f32, f32); 3]; 3] = [
    [
        (Vec2::new(-0.95, -0.45), Vec2::new(0.95, -0.45), 3., 0.1),
        (Vec2::new(0.4, 0.97), Vec2::new(0.4, -0.6), 4., 0.08),
        (Vec2::new(-0.95, 0.4), Vec2::new(0.6, 0.4), 3., 0.1),
    ],
    [
        (Vec2::new(-0.95, 0.55), Vec2::new(0.95, 0.15), 3., 0.12),
        (Vec2::new(-0.3, -0.97), Vec2::new(-0.3, 0.7), 4., 0.08),
        (Vec2::new(0.95, -0.5), Vec2::new(-0.6, -0.5), 2., 0.12),
    ],
    [
        (Vec2::new(-0.45, -0.97), Vec2::new(-0.25, 0.97), 5., 0.08),
        (Vec2::new(0.95, 0.25), Vec2::new(-0.6, 0.25), 3., 0.1),
        (Vec2::new(0.5, -0.97), Vec2::new(0.5, 0.6), 3., 0.1),
    ],
];
const STORE_EVERY: u32 = 48;
const STORE_LINE_TICKS: u32 = 32;
const STORE_SHOT_STEP: u32 = 20;
const STORE_SHOT_TICK: u32 = LINE_TICK + 2 * STORE_EVERY + STORE_SHOT_STEP;
/// The kill scene: the kills before the game line, the shot this many ticks later.
const KILL_TICK: u32 = 48;
const KILL_SHOT_TICKS: u32 = 12;
/// Seconds since the title showed, at a fixed time step per frame.
const GAME_SHOT_AT: f32 = 3.0;
const MENU_SHOT_AT: f32 = 3.2;
/// The flow scene: claim the level, die, tap to play again, then the shot.
/// The level clear fades the claims in 0.3 s ([097]) and the death takes 1 s ([038]).
const FLOW_CLEAR_AT: f32 = 1.6;
const FLOW_DIE_AT: f32 = 3.5;
const FLOW_RETRY_AT: f32 = 5.0;
const FLOW_SHOT_AT: f32 = 6.0;
const GIVE_UP_AT: f32 = 20.0;
/// One frame at 60 Hz.
const SLOW_UPDATE: Duration = Duration::from_millis(16);

/// The skins that `RIX_SKINS` names (for example `laser,brick,grid,firework`), over the
/// defaults.
pub fn forced_skins() -> Option<Skins> {
    let names = std::env::var("RIX_SKINS").ok()?;
    let mut skins = Skins::default();
    for name in names.split(',').map(str::to_uppercase) {
        if let Some(line) = LineSkin::ALL.into_iter().find(|s| s.name() == name) {
            skins.line = line;
        } else if let Some(wall) = WallSkin::ALL.into_iter().find(|s| s.name() == name) {
            skins.wall = wall;
        } else if let Some(fill) = FillSkin::ALL.into_iter().find(|s| s.name() == name) {
            skins.fill = fill;
        } else if let Some(kill) = KillSkin::ALL.into_iter().find(|s| s.name() == name) {
            skins.kill = kill;
        } else {
            warn!("RIX_SKINS: no skin {name}");
        }
    }
    Some(skins)
}

pub fn is_shot_run() -> bool {
    std::env::var_os("RIX_SHOT").is_some()
}

/// `BasePlugin` turns the renderer off for it.
pub fn is_replay_run() -> bool {
    is_shot_run() && std::env::var("RIX_SHOT_SCENE").as_deref() == Ok("replay")
}

pub(super) fn plugin(app: &mut App) {
    if let Some(seed) = std::env::var("RIX_SEED").ok().and_then(|s| s.parse().ok()) {
        let level = std::env::var("RIX_LEVEL").ok().and_then(|s| s.parse().ok());
        let clock = std::env::var("RIX_CLOCK").ok().and_then(|s| s.parse().ok());
        app.add_systems(Startup, move |mut commands: Commands| {
            new_run(&mut commands, seed);
            if let Some(secs) = clock {
                commands.insert_resource(Countdown::new(secs));
            }
            if let Some(level) = level {
                commands.insert_resource(Run {
                    level,
                    ..Run::with_seed(seed)
                });
                commands.insert_resource(GameRng::new(seed + level as u64));
            }
        });
    }
    app.insert_resource(UpdateStart(Instant::now()))
        .add_systems(First, start_update)
        .add_systems(Last, log_slow_update);

    let Ok(path) = std::env::var("RIX_SHOT") else {
        return;
    };
    let scene = match std::env::var("RIX_SHOT_SCENE").as_deref() {
        Ok("menu") => ShotScene::Menu,
        Ok("wiki") => ShotScene::Wiki,
        Ok("skins") => ShotScene::Skins,
        Ok("records") => ShotScene::Records,
        Ok("challenges") => ShotScene::Challenges,
        Ok("settings") => ShotScene::Settings,
        Ok("credits") => ShotScene::Credits,
        Ok("flow") => ShotScene::Flow,
        Ok("store") => ShotScene::Store,
        Ok("kill") => ShotScene::Kill,
        Ok("replay") => ShotScene::Replay,
        _ => ShotScene::Game,
    };
    let size = std::env::var("RIX_SHOT_SIZE").ok().and_then(|s| {
        let (w, h) = s.split_once('x')?;
        Some(UVec2::new(w.parse().ok()?, h.parse().ok()?))
    });
    let lines = std::env::var("RIX_SHOT_LINES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let fps: f64 = std::env::var("RIX_SHOT_FPS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60.);
    let frame = Duration::from_secs_f64(1. / fps);
    app.add_plugins(ScheduleRunnerPlugin::run_loop(frame))
        .insert_resource(TimeUpdateStrategy::ManualDuration(frame))
        .insert_resource(Shot {
            path: path.into(),
            scene,
            lines,
            size,
            ticks: 0,
            requested: false,
            target: default(),
            start: None,
        })
        .add_systems(Startup, (render_offscreen, pause_while_loading))
        .add_systems(OnExit(Screen::Loading), resume_after_loading)
        .add_systems(PreUpdate, drive_shot)
        .add_systems(Update, skip_menu.run_if(in_state(Screen::Title)));
    if !matches!(
        scene,
        ShotScene::Menu
            | ShotScene::Wiki
            | ShotScene::Skins
            | ShotScene::Records
            | ShotScene::Challenges
            | ShotScene::Settings
            | ShotScene::Credits
    ) {
        app.add_systems(
            FixedPreUpdate,
            drive_line.run_if(in_state(Screen::Gameplay)),
        )
        .add_systems(FixedLast, log_state.run_if(in_state(Screen::Gameplay)));
    }
    if scene == ShotScene::Kill {
        app.add_systems(FixedUpdate, drive_kill.run_if(in_state(Screen::Gameplay)));
    }
    if scene == ShotScene::Flow {
        app.add_systems(PreUpdate, drive_flow.after(drive_shot))
            .add_systems(Last, log_census);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShotScene {
    Menu,
    Wiki,
    Skins,
    Records,
    Challenges,
    Settings,
    Credits,
    Game,
    Flow,
    Store,
    Kill,
    Replay,
}

#[derive(Resource)]
struct Shot {
    path: PathBuf,
    scene: ShotScene,
    /// The store scene's line set (`RIX_SHOT_LINES`).
    lines: usize,
    size: Option<UVec2>,
    /// Game ticks since the game started.
    ticks: u32,
    requested: bool,
    /// The image the camera draws into (`render_offscreen`).
    target: Handle<Image>,
    /// Real time when the title screen first showed: the script counts from there, so the
    /// loading time does not move it ([027]).
    start: Option<f32>,
}

/// When the main schedule started this frame.
#[derive(Resource)]
struct UpdateStart(Instant);

fn start_update(mut start: ResMut<UpdateStart>) {
    start.0 = Instant::now();
}

/// The main schedule is game logic and physics, not rendering; the frame is the time since
/// the last frame.
fn log_slow_update(start: Res<UpdateStart>, time: Res<Time<Real>>) {
    let update = start.0.elapsed();
    if update > SLOW_UPDATE {
        warn!("slow update: {update:.1?} (frame {:.1?})", time.delta());
    }
}

fn skip_menu(shot: Res<Shot>, mut next_state: ResMut<NextState<Screen>>) {
    match shot.scene {
        ShotScene::Menu => {}
        ShotScene::Wiki => next_state.set(Screen::Wiki),
        ShotScene::Skins => next_state.set(Screen::Skins),
        ShotScene::Records => next_state.set(Screen::Records),
        ShotScene::Challenges => next_state.set(Screen::Challenges),
        ShotScene::Settings => next_state.set(Screen::Settings),
        ShotScene::Credits => next_state.set(Screen::Credits),
        _ => next_state.set(Screen::Gameplay),
    }
}

/// The image has scale factor 1: the UI and the touch positions use its pixels. Bevy 0.19.1 draws nothing into an image target with
/// scale factor 2 ([017] Notes), so the shot is 625 x 625, not 1250 x 1250.
fn render_offscreen(
    mut commands: Commands,
    mut shot: ResMut<Shot>,
    mut images: ResMut<Assets<Image>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<Entity, With<Camera2d>>,
) {
    let size = shot.size.unwrap_or(window.size().as_uvec2());
    let image = images.add(Image::new_target_texture(
        size.x,
        size.y,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    commands.entity(*camera).insert((
        RenderTarget::Image(ImageRenderTarget {
            handle: image.clone(),
            scale_factor: 1.,
        }),
        IsDefaultUiCamera,
    ));
    shot.target = image;
}

fn drive_shot(
    mut commands: Commands,
    mut shot: ResMut<Shot>,
    time: Res<Time<Real>>,
    mut game_time: ResMut<Time<Virtual>>,
    mut exit: MessageWriter<AppExit>,
    screen: Res<State<Screen>>,
) {
    let real = time.elapsed_secs();
    if real > GIVE_UP_AT {
        error!("RIX_SHOT: no screenshot after {GIVE_UP_AT}s, giving up");
        exit.write(AppExit::error());
        return;
    }
    if shot.requested || *screen.get() == Screen::Loading {
        return;
    }
    let now = real - *shot.start.get_or_insert(real);

    let shot_at = match shot.scene {
        ShotScene::Menu
        | ShotScene::Wiki
        | ShotScene::Skins
        | ShotScene::Records
        | ShotScene::Challenges
        | ShotScene::Settings
        | ShotScene::Credits => MENU_SHOT_AT,
        ShotScene::Game => GAME_SHOT_AT,
        ShotScene::Flow => FLOW_SHOT_AT,
        ShotScene::Store if shot.ticks < STORE_SHOT_TICK => return,
        ShotScene::Store => 0.,
        ShotScene::Kill if shot.ticks < KILL_TICK + KILL_SHOT_TICKS => return,
        ShotScene::Kill => 0.,
        ShotScene::Replay => return,
    };

    if now >= shot_at {
        shot.requested = true;
        // The capture can land a frame or two later: freeze the game so it shows this frame.
        game_time.pause();
        info!("RIX_SHOT: saving {}", shot.path.display());
        commands
            .spawn(Screenshot::image(shot.target.clone()))
            .observe(save_to_disk(shot.path.clone()))
            .observe(
                |_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                    exit.write(AppExit::Success);
                },
            );
    }
}

/// The scripted line: one touch per game tick, so every frame rate gets the same input.
fn drive_line(
    mut shot: ResMut<Shot>,
    window: Single<Entity, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    field: Res<Field>,
    mut touches: MessageWriter<TouchInput>,
    mut window_events: MessageWriter<WindowEvent>,
) {
    let tick = shot.ticks;
    shot.ticks += 1;
    let half = field.0 / 2.;
    let Some(since) = tick.checked_sub(LINE_TICK) else {
        return;
    };
    let (start, end, waves, sway, step, ticks) = if shot.scene == ShotScene::Store {
        let set = &STORE_LINES[shot.lines % STORE_LINES.len()];
        let Some(&(start, end, waves, sway)) = set.get((since / STORE_EVERY) as usize) else {
            return;
        };
        (
            start * half,
            end * half,
            waves,
            sway * half.x,
            since % STORE_EVERY,
            STORE_LINE_TICKS,
        )
    } else {
        let start = Vec2::new(-half.x + LINE_INSET.x, -half.y + LINE_INSET.y);
        let end = Vec2::new(half.x - LINE_INSET.x, -half.y + LINE_INSET.y);
        (start, end, 0., 0., since, LINE_TICKS)
    };
    if step > ticks + 1 {
        return;
    }
    let t = (step as f32 / ticks as f32).min(1.0);
    let phase = match step {
        0 => TouchPhase::Started,
        s if s > ticks => TouchPhase::Ended,
        _ => TouchPhase::Moved,
    };
    let (camera, camera_transform) = *camera;
    let normal = (end - start).normalize().perp();
    let world = (start.lerp(end, t) + normal * sway * wobble(t, waves)).extend(0.);
    let Ok(position) = camera.world_to_viewport(camera_transform, world) else {
        return;
    };
    let touch = TouchInput {
        phase,
        position,
        window: *window,
        force: None,
        id: 0,
    };
    // As winit: the line reads the ordered window events ([077]), picking both streams.
    touches.write(touch);
    window_events.write(WindowEvent::TouchInput(touch));
}

/// The kill scene: every enemy dies at `KILL_TICK`, in place order ([104]).
fn drive_kill(
    mut commands: Commands,
    shot: Res<Shot>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
) {
    if shot.ticks != KILL_TICK {
        return;
    }
    let mut enemies: Vec<_> = enemies.iter().collect();
    enemies.sort_by(|a, b| by_place(a.1.translation.truncate(), b.1.translation.truncate()));
    for (enemy, _) in enemies {
        commands.trigger(DespawnEnemy(enemy));
    }
}

/// How far a store line sways off its straight path at `t` (0 to 1), in units of its sway:
/// `waves` half waves plus a faster, smaller wobble, both zero at the ends ([103]).
fn wobble(t: f32, waves: f32) -> f32 {
    ops::sin(PI * waves * t) + ops::sin(PI * (2. * waves + 1.) * t) / 3.
}

/// A hash of the game state after `STATE_TICK` game ticks: enemy positions and velocities,
/// the free region, points, kills and the RNG state.
fn log_state(
    shot: Res<Shot>,
    mut exit: MessageWriter<AppExit>,
    enemies: Query<(&Position, &LinearVelocity), With<Enemy>>,
    free: Res<FreeRegion>,
    field: Res<Field>,
    run: Res<Run>,
    rng: Res<GameRng>,
) {
    use std::hash::{DefaultHasher, Hash, Hasher};

    if shot.ticks != STATE_TICK {
        return;
    }
    let bits = |v: Vec2| [v.x.to_bits(), v.y.to_bits()];
    let mut bodies: Vec<_> = enemies
        .iter()
        .map(|(p, v)| (bits(p.0), bits(v.0)))
        .collect();
    bodies.sort();
    let mut hasher = DefaultHasher::new();
    bodies.hash(&mut hasher);
    free.iter()
        .map(|p| bits(*p))
        .for_each(|p| p.hash(&mut hasher));
    (run.points.0, run.kill_count, rng.get_seed()).hash(&mut hasher);
    info!(
        "RIX_STATE tick {STATE_TICK}: {:016x} ({} enemies, {:.4} claimed, {} points)",
        hasher.finish(),
        bodies.len(),
        free.claimed_fraction(&field),
        run.points
    );
    if shot.scene == ShotScene::Replay {
        exit.write(AppExit::Success);
    }
}

/// Game time stands still while the assets load, so the shaders' clock (CRT, blink) starts
/// at the title screen whatever the loading time ([027]).
fn pause_while_loading(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn resume_after_loading(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}

/// The flow scene's steps, each once at its time.
fn drive_flow(
    time: Res<Time<Real>>,
    shot: Res<Shot>,
    mut step: Local<u8>,
    mut progress: ResMut<LevelProgress>,
    mut die: MessageWriter<Die>,
    window: Single<Entity, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut touches: MessageWriter<TouchInput>,
) {
    let Some(start) = shot.start else {
        return;
    };
    let now = time.elapsed_secs() - start;
    let at = [
        FLOW_CLEAR_AT,
        FLOW_DIE_AT,
        FLOW_RETRY_AT,
        FLOW_RETRY_AT + 0.05,
    ];
    if at.get(*step as usize).is_none_or(|&t| now < t) {
        return;
    }
    info!("RIX_FLOW step {} at {now:.3}", *step);
    match *step {
        0 => progress.current = 1.,
        1 => {
            die.write(Die);
        }
        _ => {
            let (camera, camera_transform) = *camera;
            let retry = Vec3::new(0., game_over::RETRY_Y, 0.);
            let Ok(position) = camera.world_to_viewport(camera_transform, retry) else {
                return;
            };
            touches.write(TouchInput {
                phase: if *step == 2 {
                    TouchPhase::Started
                } else {
                    TouchPhase::Ended
                },
                position,
                window: *window,
                force: None,
                id: 1,
            });
        }
    }
    *step += 1;
}

/// The entity census, logged when it changes.
fn log_census(
    time: Res<Time<Real>>,
    screen: Res<State<Screen>>,
    phase: Option<Res<State<Phase>>>,
    all: Query<(), Without<IsResource>>,
    enemies: Query<(), With<Enemy>>,
    items: Query<(), With<Item>>,
    walls: Query<(), With<Wall>>,
    shapes: Query<(), With<DrawnShape>>,
    territories: Query<(), With<Territory>>,
    lines: Query<(), With<TempLineSegment>>,
    texts: Query<(), Or<(With<Text>, With<Text2d>)>>,
    run: Res<Run>,
    mut last: Local<String>,
) {
    let census = format!(
        "{:?} {:?} level {}: {} entities, {} enemies, {} items, {} walls, {} shapes, {} territories, {} lines, {} texts",
        screen.get(),
        phase.map(|p| *p.get()),
        run.level,
        all.iter().count(),
        enemies.iter().count(),
        items.iter().count(),
        walls.iter().count(),
        shapes.iter().count(),
        territories.iter().count(),
        lines.iter().count(),
        texts.iter().count(),
    );
    if *last != census {
        info!("RIX_FLOW {census} (t {:.3})", time.elapsed_secs());
        *last = census;
    }
}
