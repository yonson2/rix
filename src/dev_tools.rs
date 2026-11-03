//! Dev-only tools, compiled with the `dev_tools` feature ([006]). Release builds never
//! contain them.
//!
//! - `RIX_SEED=<u64>`: the seed of the first run.
//! - `RIX_SHOT=<file.png>`: skip the menu, draw one line across the bottom of the field
//!   with touch input, save a screenshot to the file and exit. Each frame advances time by
//!   a fixed step, so the run does not depend on the frame rate ([062]).
//! - `RIX_SHOT_SCENE=menu`: with `RIX_SHOT`, screenshot the menu instead.
//! - `RIX_SHOT_SCENE=flow`: with `RIX_SHOT`, the game line, then a level clear, a death
//!   and a retry; logs an entity census (`RIX_FLOW`) each time it changes ([026]).
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
    enemies::Enemy,
    events::Die,
    items::Item,
    polygon::{DrawnShape, TempLineSegment, Territory},
    setup::{GameRng, LevelProgress, Run, new_run},
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
/// Seconds since the title showed, at a fixed time step per frame.
const GAME_SHOT_AT: f32 = 3.0;
const MENU_SHOT_AT: f32 = 3.2;
/// The flow scene: claim the level, die, tap to play again, then the shot.
/// The level clear banner takes 1.2 s and the death 1 s ([038]).
const FLOW_CLEAR_AT: f32 = 1.6;
const FLOW_DIE_AT: f32 = 3.5;
const FLOW_RETRY_AT: f32 = 5.0;
const FLOW_SHOT_AT: f32 = 6.0;
const GIVE_UP_AT: f32 = 20.0;
/// One frame at 60 Hz.
const SLOW_UPDATE: Duration = Duration::from_millis(16);

pub fn is_shot_run() -> bool {
    std::env::var_os("RIX_SHOT").is_some()
}

/// `BasePlugin` turns the renderer off for it.
pub fn is_replay_run() -> bool {
    is_shot_run() && std::env::var("RIX_SHOT_SCENE").as_deref() == Ok("replay")
}

pub(super) fn plugin(app: &mut App) {
    if let Some(seed) = std::env::var("RIX_SEED").ok().and_then(|s| s.parse().ok()) {
        app.add_systems(Startup, move |mut commands: Commands| {
            new_run(&mut commands, seed);
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
        Ok("flow") => ShotScene::Flow,
        Ok("replay") => ShotScene::Replay,
        _ => ShotScene::Game,
    };
    let size = std::env::var("RIX_SHOT_SIZE").ok().and_then(|s| {
        let (w, h) = s.split_once('x')?;
        Some(UVec2::new(w.parse().ok()?, h.parse().ok()?))
    });
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
    if scene != ShotScene::Menu {
        app.add_systems(
            FixedPreUpdate,
            drive_line.run_if(in_state(Screen::Gameplay)),
        )
        .add_systems(FixedLast, log_state.run_if(in_state(Screen::Gameplay)));
    }
    if scene == ShotScene::Flow {
        app.add_systems(PreUpdate, drive_flow.after(drive_shot))
            .add_systems(Last, log_census);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShotScene {
    Menu,
    Game,
    Flow,
    Replay,
}

#[derive(Resource)]
struct Shot {
    path: PathBuf,
    scene: ShotScene,
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
    if shot.scene != ShotScene::Menu {
        next_state.set(Screen::Gameplay);
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
        ShotScene::Menu => MENU_SHOT_AT,
        ShotScene::Game => GAME_SHOT_AT,
        ShotScene::Flow => FLOW_SHOT_AT,
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
    let Some(step) = tick.checked_sub(LINE_TICK).filter(|&s| s <= LINE_TICKS + 1) else {
        return;
    };
    let t = (step as f32 / LINE_TICKS as f32).min(1.0);
    let phase = match step {
        0 => TouchPhase::Started,
        s if s > LINE_TICKS => TouchPhase::Ended,
        _ => TouchPhase::Moved,
    };
    let (camera, camera_transform) = *camera;
    let half = field.0 / 2.;
    let start = Vec2::new(-half.x + LINE_INSET.x, -half.y + LINE_INSET.y);
    let end = Vec2::new(half.x - LINE_INSET.x, -half.y + LINE_INSET.y);
    let world = start.lerp(end, t).extend(0.);
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
