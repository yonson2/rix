use avian2d::prelude::*;
use bevy::{color::palettes::css::BLACK, prelude::*, window::WindowResolution};

use crate::{camera, crt, game::wall::FIELD};

/// `BasePlugin` sets sane defaults, A combination
/// of `DefaultPlugin`s configurations and the
/// camera.
pub(super) fn plugin(app: &mut App) {
    let primary_window = Window {
        title: "Rix".into(),
        // The camera fits the fixed field into any size ([019]).
        resolution: WindowResolution::new(FIELD.x as u32, FIELD.y as u32),
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
    app.add_plugins(PhysicsPlugins::default());
    // avian 0.3 sends collision events only for colliders that ask for them.
    app.register_required_components::<Collider, CollisionEventsEnabled>();
    app.add_plugins((camera::plugin, crt::plugin));
}
