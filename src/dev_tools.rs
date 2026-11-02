//! Dev-only tools, compiled with the `dev_tools` feature ([006]). Release builds never
//! contain them.
//!
//! - `RIX_SEED=<u64>`: the seed of the first run.
//! - `RIX_SHOT=<file.png>`: skip the menu, draw one line across the bottom of the field
//!   with touch input, save a screenshot to the file and exit. Each frame advances time by
//!   a fixed step, so the run does not depend on the frame rate ([062]).
//! - `RIX_SHOT_SCENE=menu`: with `RIX_SHOT`, screenshot the menu instead.
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
    input::touch::TouchPhase,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    },
    time::TimeUpdateStrategy,
    window::PrimaryWindow,
};

use crate::rix::{setup::LevelInfo, state::AppState, wall::FIELD};

/// The scripted line, in world units: from inside the left wall to inside the right wall,
/// a little above the bottom wall.
const LINE_START: Vec2 = Vec2::new(12.5 - FIELD.x / 2., 87.5 - FIELD.y / 2.);
const LINE_END: Vec2 = Vec2::new(FIELD.x / 2. - 12.5, 87.5 - FIELD.y / 2.);
const LINE_FRAMES: u32 = 12;
/// Seconds since startup, at a fixed 1/60 s per frame.
const LINE_AT: f32 = 1.0;
const GAME_SHOT_AT: f32 = 3.0;
const MENU_SHOT_AT: f32 = 3.2;
const GIVE_UP_AT: f32 = 20.0;
/// One frame at 60 Hz.
const SLOW_UPDATE: Duration = Duration::from_millis(16);

pub struct DevToolsPlugin;

pub fn is_shot_run() -> bool {
    std::env::var_os("RIX_SHOT").is_some()
}

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
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
            .add_systems(Update, skip_menu.run_if(in_state(AppState::Menu)));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShotScene {
    Menu,
    Game,
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

fn skip_menu(shot: Res<Shot>, mut next_state: ResMut<NextState<AppState>>) {
    if shot.scene == ShotScene::Game {
        next_state.set(AppState::InGame);
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
        ShotScene::Game => {
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
            GAME_SHOT_AT
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
