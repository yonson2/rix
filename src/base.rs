use bevy::{color::palettes::css::BLACK, prelude::*, window::WindowResolution};

use crate::camera::CameraPlugin;

/// `BasePlugin` sets sane defaults, A combination
/// of `DefaultPlugin`s configurations and the
/// camera.
pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut App) {
        let primary_window = Window {
            title: "Rix".into(),
            resizable: false,
            resolution: WindowResolution::new(1000., 1000.).with_scale_factor_override(1.0),
            // canvas: Some("#rix".to_owned()), TODO:
            // https://github.com/nolantait/bevy-starter/blob/master/web
            desired_maximum_frame_latency: core::num::NonZero::new(1u32),
            ..default()
        };

        app.insert_resource(ClearColor(BLACK.into()));
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window),
            ..default()
        }));
        app.add_plugins(CameraPlugin);
    }
}
