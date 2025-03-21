use bevy::{prelude::*, window::PrimaryWindow};

pub struct CameraPlugin;

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn get_cursor_world_position(
    q_windows: &Query<&Window, With<PrimaryWindow>>,
    q_camera: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = q_windows.single();
    let (camera, cam_transform) = q_camera.single();

    window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(cam_transform, cursor).ok())
        .map(|wp| wp.origin.truncate())
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
    }
}
