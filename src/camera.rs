use bevy::{
    anti_alias::smaa::Smaa, core_pipeline::tonemapping::Tonemapping, post_process::bloom::Bloom,
    prelude::*, render::view::Hdr, window::PrimaryWindow,
};

use crate::crt::PostProcessSettings;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Hdr,
        Tonemapping::TonyMcMapface,
        Bloom::default(),
        Msaa::Off,
        Smaa::default(),
        PostProcessSettings {
            intensity: 1., // Controls overall CRT effect strength
            ambient_light: 0.5,
            time: 0.0, // Will be updated each frame
        },
    ));
}

pub fn get_cursor_world_position(
    window: Single<&Window, With<PrimaryWindow>>,
    q_camera: Single<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let (camera, cam_transform) = q_camera.into_inner();

    window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(cam_transform, cursor).ok())
        .map(|wp| wp.origin.truncate())
}
