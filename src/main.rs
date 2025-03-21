use bevy::{
    prelude::*,
    window::WindowResolution,
};
use bevy_rapier2d::prelude::*;

mod camera;
use camera::CameraPlugin;

mod chain;
use chain::ChainPlugin;

mod snake;
use snake::SnakePlugin;

const CAMERA_SPEED: f32 = 500.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(1000., 1000.).with_scale_factor_override(1.0),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(CameraPlugin {
            speed: CAMERA_SPEED,
        })
        .add_plugins(ChainPlugin)
        .add_plugins(SnakePlugin)
        .run();
}
