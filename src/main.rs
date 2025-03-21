use avian2d::prelude::*;
use bevy::{prelude::*, window::WindowResolution};
use rix::{ball::BallPlugin, camera::CameraPlugin, serpent::SerpentPlugin, wall::WallPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(1000., 1000.).with_scale_factor_override(1.0),
            resizable: false,
            ..default()
        }),
        ..default()
    }));

    app.add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec2::ZERO));
    app.add_plugins(CameraPlugin);
    app.add_plugins(BallPlugin);
    app.add_plugins(WallPlugin);
    app.add_plugins(SerpentPlugin);
    app.run();
}
