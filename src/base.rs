use avian2d::prelude::*;
use bevy::{color::palettes::css::BLACK, prelude::*, window::WindowResolution};

use crate::{camera::CameraPlugin, crt::CrtPlugin};

/// `BasePlugin` sets sane defaults, A combination
/// of `DefaultPlugin`s configurations and the
/// camera.
pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut App) {
        let primary_window = Window {
            title: "Rix".into(),
            resizable: false,
            resolution: WindowResolution::new(625., 625.),
            ..default()
        };

        app.insert_resource(ClearColor(BLACK.into()));
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window),
            ..default()
        }));
        app.add_plugins(PhysicsPlugins::default());
        app.add_plugins(PhysicsPickingPlugin);
        app.add_plugins(CameraPlugin);
        app.add_plugins(CrtPlugin);
    }
}
