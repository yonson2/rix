use avian2d::prelude::*;
use bevy::{
    app::PluginGroupBuilder, color::palettes::css::BLACK, prelude::*, window::WindowResolution,
};

use crate::{camera, crt, game::wall::Field};

/// `BasePlugin` sets sane defaults, A combination
/// of `DefaultPlugin`s configurations and the
/// camera.
pub(super) fn plugin(app: &mut App) {
    let primary_window = Window {
        title: "Rix".into(),
        // The default field size; the camera fits the field into any size ([019], [080]).
        resolution: WindowResolution::new(Field::default().x as u32, Field::default().y as u32),
        // A sized window on iOS draws a surface larger than the screen ([020]).
        #[cfg(target_os = "ios")]
        mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
        // Swipes from the screen edges draw lines, not system gestures ([020]).
        #[cfg(target_os = "ios")]
        preferred_screen_edges_deferring_system_gestures: bevy::window::ScreenEdge::All,
        #[cfg(target_os = "ios")]
        prefers_status_bar_hidden: true,
        #[cfg(target_os = "ios")]
        prefers_home_indicator_hidden: true,
        ..default()
    };

    app.insert_resource(ClearColor(BLACK.into()));
    let plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(primary_window),
        ..default()
    });
    // The emulator's Vulkan driver crashes on debug object names ([022]).
    #[cfg(target_os = "android")]
    let plugins = {
        use bevy::render::settings::{InstanceFlags, WgpuSettings};
        let mut wgpu = WgpuSettings::default();
        wgpu.instance_flags.remove(InstanceFlags::DEBUG);
        plugins.set(bevy::render::RenderPlugin {
            render_creation: wgpu.into(),
            ..default()
        })
    };
    // A shot run draws into an image and opens no window ([064]).
    #[cfg(feature = "dev_tools")]
    let plugins = if crate::dev_tools::is_shot_run() {
        plugins.disable::<bevy::winit::WinitPlugin>()
    } else {
        plugins
    };
    app.add_plugins(plugins);
    app.add_plugins(physics());
    // avian 0.3 sends collision events only for colliders that ask for them.
    app.register_required_components::<Collider, CollisionEventsEnabled>();
    app.add_plugins((camera::plugin, crt::plugin));
}

/// Physics steps at 64 Hz; each body's `Transform` is interpolated between the steps, so it
/// moves the same distance every frame at any frame rate ([076]). Gameplay still reads the
/// stepped values: the interpolation puts them back before each step.
fn physics() -> PluginGroupBuilder {
    PhysicsPlugins::default().set(PhysicsInterpolationPlugin::interpolate_all())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    /// The distance a body at constant speed moves in each frame at 60 fps, after a warm-up.
    fn frame_steps(physics: PluginGroupBuilder) -> Vec<f32> {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::mesh::MeshPlugin,
            TransformPlugin,
            physics,
        ))
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f64(1. / 60.),
        ));
        app.finish();
        let body = app
            .world_mut()
            .spawn((
                RigidBody::Dynamic,
                GravityScale(0.),
                Collider::circle(10.),
                LinearVelocity(Vec2::new(300., 0.)),
                Transform::default(),
            ))
            .id();
        let mut xs = vec![];
        for _ in 0..70 {
            app.update();
            xs.push(app.world().get::<Transform>(body).unwrap().translation.x);
        }
        xs[6..].windows(2).map(|w| w[1] - w[0]).collect()
    }

    #[test]
    fn bodies_move_evenly_at_60_fps_with_64_hz_physics() {
        let spread = |steps: &[f32]| {
            let (min, max) = steps
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), &s| (a.min(s), b.max(s)));
            max - min
        };
        let stepped = frame_steps(PhysicsPlugins::default().build());
        assert!(
            spread(&stepped) > 4.,
            "a frame with two steps jumps: {stepped:?}"
        );
        let smooth = frame_steps(physics());
        assert!(spread(&smooth) < 0.5, "{smooth:?}");
    }
}
