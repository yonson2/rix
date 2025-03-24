use bevy::{prelude::*, window::WindowResolution};
use rix::{camera::CameraPlugin, rix::RixPlugin};

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

    app.add_plugins(CameraPlugin);
    app.add_plugins(RixPlugin);
    app.run();
}
