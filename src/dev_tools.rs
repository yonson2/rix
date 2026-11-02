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
//! - `RIX_SHOT_SIZE=<w>x<h>`: with `RIX_SHOT`, the image size (default: the window size).
//!   The field letterboxes into any size ([019]).
//! - Always: a warning for each update over 16 ms ([024]).
//!
//! A `RIX_SHOT` run opens no window and does not take the focus: `BasePlugin` leaves out
//! winit, the schedule runner drives the frames and the camera draws into an image ([064]).

use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

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
    window::PrimaryWindow,
};

use crate::game::{
    enemies::Enemy,
    events::Die,
    items::Item,
    polygon::{DrawnShape, PathSegment, TempLineSegment, Territory},
    setup::LevelInfo,
    state::Phase,
    wall::{FIELD, Wall},
};
use crate::screens::Screen;

/// The scripted line, in world units: from inside the left wall to inside the right wall,
/// a little above the bottom wall.
const LINE_START: Vec2 = Vec2::new(12.5 - FIELD.x / 2., 87.5 - FIELD.y / 2.);
const LINE_END: Vec2 = Vec2::new(FIELD.x / 2. - 12.5, 87.5 - FIELD.y / 2.);
const LINE_FRAMES: u32 = 12;
/// Seconds since startup, at a fixed 1/60 s per frame.
const LINE_AT: f32 = 1.0;
const GAME_SHOT_AT: f32 = 3.0;
const MENU_SHOT_AT: f32 = 3.2;
/// The flow scene: claim the level, die, tap to play again, then the shot.
const FLOW_CLEAR_AT: f32 = 1.6;
const FLOW_DIE_AT: f32 = 3.0;
const FLOW_RETRY_AT: f32 = 3.5;
const FLOW_SHOT_AT: f32 = 4.5;
const GIVE_UP_AT: f32 = 20.0;
/// One frame at 60 Hz.
const SLOW_UPDATE: Duration = Duration::from_millis(16);

pub fn is_shot_run() -> bool {
    std::env::var_os("RIX_SHOT").is_some()
}

pub(super) fn plugin(app: &mut App) {
    if let Some(seed) = std::env::var("RIX_SEED").ok().and_then(|s| s.parse().ok()) {
        app.add_systems(Startup, move |mut level_info: ResMut<LevelInfo>| {
            *level_info = LevelInfo::with_seed(seed);
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
        _ => ShotScene::Game,
    };
    let size = std::env::var("RIX_SHOT_SIZE").ok().and_then(|s| {
        let (w, h) = s.split_once('x')?;
        Some(UVec2::new(w.parse().ok()?, h.parse().ok()?))
    });
    let frame = Duration::from_secs_f64(1. / 60.);
    app.add_plugins(ScheduleRunnerPlugin::run_loop(frame))
        .insert_resource(TimeUpdateStrategy::ManualDuration(frame))
        .insert_resource(Shot {
            path: path.into(),
            scene,
            size,
            line_frame: 0,
            requested: false,
            target: default(),
        })
        .add_systems(Startup, render_offscreen)
        .add_systems(PreUpdate, drive_shot)
        .add_systems(Update, skip_menu.run_if(in_state(Screen::Title)));
    if scene == ShotScene::Flow {
        app.add_systems(PreUpdate, drive_flow)
            .add_systems(Last, log_census);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShotScene {
    Menu,
    Game,
    Flow,
}

#[derive(Resource)]
struct Shot {
    path: PathBuf,
    scene: ShotScene,
    size: Option<UVec2>,
    line_frame: u32,
    requested: bool,
    /// The image the camera draws into (`render_offscreen`).
    target: Handle<Image>,
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

#[allow(clippy::too_many_arguments)]
fn drive_shot(
    mut commands: Commands,
    mut shot: ResMut<Shot>,
    time: Res<Time<Real>>,
    mut game_time: ResMut<Time<Virtual>>,
    window: Single<Entity, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut touches: MessageWriter<TouchInput>,
    mut exit: MessageWriter<AppExit>,
) {
    let now = time.elapsed_secs();
    if now > GIVE_UP_AT {
        error!("RIX_SHOT: no screenshot after {GIVE_UP_AT}s, giving up");
        exit.write(AppExit::error());
        return;
    }
    if shot.requested {
        return;
    }

    let shot_at = match shot.scene {
        ShotScene::Menu => MENU_SHOT_AT,
        ShotScene::Game | ShotScene::Flow => {
            if now >= LINE_AT && shot.line_frame <= LINE_FRAMES + 1 {
                let frame = shot.line_frame;
                let t = (frame as f32 / LINE_FRAMES as f32).min(1.0);
                let phase = match frame {
                    0 => TouchPhase::Started,
                    f if f > LINE_FRAMES => TouchPhase::Ended,
                    _ => TouchPhase::Moved,
                };
                let (camera, camera_transform) = *camera;
                let world = LINE_START.lerp(LINE_END, t).extend(0.);
                let Ok(position) = camera.world_to_viewport(camera_transform, world) else {
                    return;
                };
                touches.write(TouchInput {
                    phase,
                    position,
                    window: *window,
                    force: None,
                    id: 0,
                });
                shot.line_frame += 1;
            }
            if shot.scene == ShotScene::Flow {
                FLOW_SHOT_AT
            } else {
                GAME_SHOT_AT
            }
        }
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

/// The flow scene's steps, each once at its time.
fn drive_flow(
    time: Res<Time<Real>>,
    mut step: Local<u8>,
    mut level_info: ResMut<LevelInfo>,
    mut die: MessageWriter<Die>,
    window: Single<Entity, With<PrimaryWindow>>,
    mut touches: MessageWriter<TouchInput>,
) {
    let now = time.elapsed_secs();
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
        0 => level_info.current = 1.,
        1 => {
            die.write(Die);
        }
        _ => {
            touches.write(TouchInput {
                phase: if *step == 2 {
                    TouchPhase::Started
                } else {
                    TouchPhase::Ended
                },
                position: Vec2::new(100., 100.),
                window: *window,
                force: None,
                id: 1,
            });
        }
    }
    *step += 1;
}

/// The entity census, logged when it changes.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
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
    lines: Query<(), Or<(With<TempLineSegment>, With<PathSegment>)>>,
    texts: Query<(), Or<(With<Text>, With<Text2d>)>>,
    level_info: Res<LevelInfo>,
    mut last: Local<String>,
) {
    let census = format!(
        "{:?} {:?} level {}: {} entities, {} enemies, {} items, {} walls, {} shapes, {} territories, {} lines, {} texts",
        screen.get(),
        phase.map(|p| *p.get()),
        level_info.level,
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
