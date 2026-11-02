//! Dev-only tools, compiled with the `dev_tools` feature ([006]). Release builds never
//! contain them.
//!
//! - `RIX_SEED=<u64>`: the seed of the first run.
//! - `RIX_SHOT=<file.png>`: skip the menu, draw one line across the bottom of the field
//!   with touch input, save a screenshot to the file and exit. Each frame advances time by
//!   a fixed step, so the run does not depend on the frame rate ([062]).
//! - `RIX_SHOT_SCENE=menu`: with `RIX_SHOT`, screenshot the menu instead.
//!
//! A `RIX_SHOT` run opens no window and does not take the focus: `BasePlugin` leaves out
//! winit, the schedule runner drives the frames and the camera draws into an image ([064]).
//! The primary `Window` still exists, with no OS window behind it, so the game reads its
//! size as usual.

use std::{path::PathBuf, time::Duration};

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

use crate::rix::{setup::LevelInfo, state::AppState};

/// The scripted line, in window coordinates as a fraction of the window size: from inside
/// the left wall to inside the right wall, a little above the bottom wall.
const LINE_START: Vec2 = Vec2::new(0.02, 0.86);
const LINE_END: Vec2 = Vec2::new(0.98, 0.86);
const LINE_FRAMES: u32 = 12;
/// Seconds since startup, at a fixed 1/60 s per frame.
const LINE_AT: f32 = 1.0;
const GAME_SHOT_AT: f32 = 3.0;
const MENU_SHOT_AT: f32 = 3.2;
const GIVE_UP_AT: f32 = 20.0;

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

        let Ok(path) = std::env::var("RIX_SHOT") else {
            return;
        };
        let scene = match std::env::var("RIX_SHOT_SCENE").as_deref() {
            Ok("menu") => ShotScene::Menu,
            _ => ShotScene::Game,
        };
        let frame = Duration::from_secs_f64(1. / 60.);
        app.add_plugins(ScheduleRunnerPlugin::run_loop(frame))
            .insert_resource(TimeUpdateStrategy::ManualDuration(frame))
            .insert_resource(Shot {
                path: path.into(),
                scene,
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
    line_frame: u32,
    requested: bool,
    /// The image the camera draws into (`render_offscreen`).
    target: Handle<Image>,
}

fn skip_menu(shot: Res<Shot>, mut next_state: ResMut<NextState<AppState>>) {
    if shot.scene == ShotScene::Game {
        next_state.set(AppState::InGame);
    }
}

/// The image has the window's logical size at scale 1, so the world, the UI and the touch
/// positions map as on the window. Bevy 0.19.1 draws nothing into an image target with
/// scale factor 2 ([017] Notes), so the shot is 625 x 625, not 1250 x 1250.
fn render_offscreen(
    mut commands: Commands,
    mut shot: ResMut<Shot>,
    mut images: ResMut<Assets<Image>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<Entity, With<Camera2d>>,
) {
    let size = window.size().as_uvec2();
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
    window: Single<(Entity, &Window), With<PrimaryWindow>>,
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

    let (window_entity, window) = *window;
    let shot_at = match shot.scene {
        ShotScene::Menu => MENU_SHOT_AT,
        ShotScene::Game => {
            if now >= LINE_AT && shot.line_frame <= LINE_FRAMES + 1 {
                let size = window.size();
                let frame = shot.line_frame;
                let t = (frame as f32 / LINE_FRAMES as f32).min(1.0);
                let phase = match frame {
                    0 => TouchPhase::Started,
                    f if f > LINE_FRAMES => TouchPhase::Ended,
                    _ => TouchPhase::Moved,
                };
                touches.write(TouchInput {
                    phase,
                    position: LINE_START.lerp(LINE_END, t) * size,
                    window: window_entity,
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
