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
            resolution: WindowResolution::new(625, 625),
            ..default()
        };

        app.insert_resource(ClearColor(BLACK.into()));
        let plugins = DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window),
            ..default()
        });
        // A shot run draws into an image and opens no window ([064]).
        #[cfg(feature = "dev_tools")]
        let plugins = if crate::dev_tools::is_shot_run() {
            plugins.disable::<bevy::winit::WinitPlugin>()
        } else {
            plugins
        };
        app.add_plugins(plugins);
        app.add_plugins(PhysicsPlugins::default());
        // avian 0.3 sends collision events only for colliders that ask for them.
        app.register_required_components::<Collider, CollisionEventsEnabled>();
        app.add_plugins(PhysicsPickingPlugin);
        app.add_plugins(CameraPlugin);
        app.add_plugins(CrtPlugin);
    }
}
