//! Dev-only tools, compiled with the `dev_tools` feature ([006]). Release builds never
//! contain them.
//!
//! - `RIX_SEED=<u64>`: the seed of the first run.
//! - `RIX_SHOT=<file.png>`: skip the menu, draw one line across the bottom of the field
//!   with touch input, save a screenshot of the window to the file and exit. Each frame
//!   advances time by a fixed step, so the run does not depend on the frame rate ([062]).
//! - `RIX_SHOT_SCENE=menu`: with `RIX_SHOT`, screenshot the menu instead.
//!
//! On macOS a `RIX_SHOT` run gives the focus back to the app that had it and hides itself,
//! so it does not take over the keyboard of whoever runs it.

use std::{path::PathBuf, time::Duration};

use bevy::{
    input::touch::TouchPhase,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
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
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1. / 60.,
        )))
        .insert_resource(Shot {
            path: path.into(),
            scene,
            line_frame: 0,
            requested: false,
        })
        .add_systems(PreUpdate, drive_shot)
        .add_systems(Update, skip_menu.run_if(in_state(AppState::Menu)));
        #[cfg(target_os = "macos")]
        // SAFETY: AppKit calls; plugins build on the main thread.
        if let Some(front) =
            unsafe { objc2_app_kit::NSWorkspace::sharedWorkspace().frontmostApplication() }
        {
            app.insert_resource(FocusOwner(unsafe { front.processIdentifier() }))
                .add_systems(Last, give_focus_back);
        }
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
}

fn skip_menu(shot: Res<Shot>, mut next_state: ResMut<NextState<AppState>>) {
    if shot.scene == ShotScene::Game {
        next_state.set(AppState::InGame);
    }
}

fn drive_shot(
    mut commands: Commands,
    mut shot: ResMut<Shot>,
    time: Res<Time<Real>>,
    mut game_time: ResMut<Time<Virtual>>,
    window: Single<(Entity, &Window), With<PrimaryWindow>>,
    mut touches: EventWriter<TouchInput>,
    mut exit: EventWriter<AppExit>,
) {
    let now = time.elapsed_secs();
    if now > GIVE_UP_AT {
        error!("RIX_SHOT: no screenshot after {GIVE_UP_AT}s, giving up");
        exit.send(AppExit::error());
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
                touches.send(TouchInput {
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
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(shot.path.clone()))
            .observe(
                |_: Trigger<ScreenshotCaptured>, mut exit: EventWriter<AppExit>| {
                    exit.send(AppExit::Success);
                },
            );
    }
}

/// The process that was in front when the run started.
#[cfg(target_os = "macos")]
#[derive(Resource)]
struct FocusOwner(i32);

/// winit activates the app at launch and Bevy 0.15 has no setting to stop it. An exclusive
/// system runs on the main thread, which AppKit needs. The window still renders while the
/// app is hidden.
#[cfg(target_os = "macos")]
fn give_focus_back(world: &mut World) {
    use objc2_app_kit::{NSApplication, NSApplicationActivationOptions, NSRunningApplication};
    use objc2_foundation::MainThreadMarker;

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    // SAFETY: AppKit calls on the main thread.
    unsafe {
        if !app.isActive() {
            return;
        }
        let owner = world.resource::<FocusOwner>().0;
        if let Some(owner) = NSRunningApplication::runningApplicationWithProcessIdentifier(owner) {
            owner.activateWithOptions(NSApplicationActivationOptions::empty());
        }
    }
    app.hide(None);
}
