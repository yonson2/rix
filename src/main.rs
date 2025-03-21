use avian2d::prelude::*;
use bevy::{prelude::*, window::WindowResolution};

mod camera;
use camera::CameraPlugin;

mod chain;
use chain::ChainPlugin;

mod snake;
use snake::SnakePlugin;

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
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec2::ZERO))
        .add_plugins(CameraPlugin)
        .add_plugins(ChainPlugin)
        .add_plugins(SnakePlugin)
        .run();
}
