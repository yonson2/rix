use bevy::{
    anti_alias::smaa::Smaa,
    camera::{Hdr, ScalingMode},
    core_pipeline::tonemapping::Tonemapping,
    post_process::bloom::Bloom,
    prelude::*,
};

use crate::{crt::PostProcessSettings, game::wall::Field};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PreStartup, spawn_camera)
        .add_systems(Update, fit_camera.run_if(resource_changed::<Field>));
}

/// The whole field fits any window; the rest is letterbox ([019], [080]).
fn projection(field: &Field) -> Projection {
    Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::AutoMin {
            min_width: field.x,
            min_height: field.y,
        },
        ..OrthographicProjection::default_2d()
    })
}

fn fit_camera(field: Res<Field>, mut camera: Single<&mut Projection, With<Camera2d>>) {
    **camera = projection(&field);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        projection(&Field::default()),
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
