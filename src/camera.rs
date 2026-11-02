use bevy::{
    anti_alias::smaa::Smaa,
    camera::{Hdr, ScalingMode},
    core_pipeline::tonemapping::Tonemapping,
    post_process::bloom::Bloom,
    prelude::*,
};

use crate::{crt::PostProcessSettings, game::wall::FIELD};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PreStartup, spawn_camera);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        // The whole field fits any window; the rest is letterbox ([019]).
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: FIELD.x,
                min_height: FIELD.y,
            },
            ..OrthographicProjection::default_2d()
        }),
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
