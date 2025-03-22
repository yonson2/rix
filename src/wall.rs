use bevy::{
    color::palettes::tailwind::{GRAY_500, GRAY_600},
    math::VectorSpace,
    prelude::*,
};
use std::time::Duration;

use crate::player::WallTransitionEvent;

pub const WALL_THICKNESS: f32 = 15.;
pub const WALL_MARGIN: f32 = 20.;
pub struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_walls)
            .add_systems(Update, (resize_walls, handle_wall_transitions));
    }
}

#[derive(Component)]
pub struct Border {
    ///0=top, 1=right, 2=bottom, 3=left
    side: usize,
}

#[derive(Resource)]
struct WallFlashTimer {
    timer: Timer,
    is_active: bool,
    progress: f32,
    duration: f32,
}

impl Default for WallFlashTimer {
    fn default() -> Self {
        let duration_ms = 1000.0;
        Self {
            timer: Timer::new(Duration::from_millis(duration_ms as u64), TimerMode::Once),
            is_active: false,
            progress: 0.0,
            duration: duration_ms / 1000.0,
        }
    }
}

pub fn spawn_walls(
    mut commands: Commands,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.single();
    let (width, height) = (window.width(), window.height());

    let playable_width = width - 2.0 * WALL_MARGIN;
    let playable_height = height - 2.0 * WALL_MARGIN;

    let half_playable_width = playable_width / 2.0;
    let half_playable_height = playable_height / 2.0;

    let wall_material = materials.add(Color::from(GRAY_600));

    // Top
    commands.spawn((
        Border { side: 0 },
        Mesh2d(meshes.add(Rectangle::new(playable_width, WALL_THICKNESS))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(0.0, half_playable_height - WALL_THICKNESS / 2.0, 1.0),
    ));

    // Right
    commands.spawn((
        Border { side: 1 },
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, playable_height))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(half_playable_width - WALL_THICKNESS / 2.0, 0.0, 1.0),
    ));

    // Bottom
    commands.spawn((
        Border { side: 2 },
        Mesh2d(meshes.add(Rectangle::new(playable_width, WALL_THICKNESS))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(0.0, -half_playable_height + WALL_THICKNESS / 2.0, 1.0),
    ));

    // Left
    commands.spawn((
        Border { side: 3 },
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, playable_height))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(-half_playable_width + WALL_THICKNESS / 2.0, 0.0, 1.0),
    ));

    commands.init_resource::<WallFlashTimer>();
}

fn resize_walls(
    mut borders: Query<(&Border, &mut Transform, &mut Mesh2d)>,
    windows: Query<&Window, Changed<Window>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let (width, height) = (window.width(), window.height());

    let playable_width = width - 2.0 * WALL_MARGIN;
    let playable_height = height - 2.0 * WALL_MARGIN;

    let half_playable_width = playable_width / 2.0;
    let half_playable_height = playable_height / 2.0;

    for (border, mut transform, mut mesh2d) in &mut borders {
        match border.side {
            0 => {
                // Top
                transform.translation.x = 0.0;
                transform.translation.y = half_playable_height - WALL_THICKNESS / 2.0;
                mesh2d.0 = meshes.add(Rectangle::new(playable_width, WALL_THICKNESS));
            }
            1 => {
                // Right
                transform.translation.x = half_playable_width - WALL_THICKNESS / 2.0;
                transform.translation.y = 0.0;
                mesh2d.0 = meshes.add(Rectangle::new(WALL_THICKNESS, playable_height));
            }
            2 => {
                // Bottom
                transform.translation.x = 0.0;
                transform.translation.y = -half_playable_height + WALL_THICKNESS / 2.0;
                mesh2d.0 = meshes.add(Rectangle::new(playable_width, WALL_THICKNESS));
            }
            3 => {
                // Left
                transform.translation.x = -half_playable_width + WALL_THICKNESS / 2.0;
                transform.translation.y = 0.0;
                mesh2d.0 = meshes.add(Rectangle::new(WALL_THICKNESS, playable_height));
            }
            _ => {}
        }
    }
}

fn handle_wall_transitions(
    mut wall_transition_events: EventReader<WallTransitionEvent>,
    borders: Query<(Entity, &Border, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut flash_timer: ResMut<WallFlashTimer>,
    time: Res<Time>,
) {
    if flash_timer.is_active {
        flash_timer.timer.tick(time.delta());

        // Calculate progress as a value from 0.0 to 1.0
        flash_timer.progress = flash_timer.timer.elapsed_secs() / flash_timer.duration;

        if flash_timer.timer.finished() {
            for (_, _, material_handle) in &borders {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.color = Color::from(GRAY_600);
                }
            }

            flash_timer.is_active = false;
        } else {
            // Use linear interpolation between colors for a smooth transition
            let start_color = Oklaba::from(GRAY_500);
            let end_color = Oklaba::from(GRAY_600);

            // Use cubic-bezier-like easing instead of linear
            let t = flash_timer.progress;
            // Apply a simple easing function (cubic ease-out)
            let eased_t = 1.0 - (1.0 - t).powi(3);

            let current_color = start_color.lerp(end_color, eased_t);

            for (_, _, material_handle) in &borders {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.color = current_color.into();
                }
            }
        }
    }

    for _ in wall_transition_events.read() {
        for (_, _, material_handle) in &borders {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.color = Color::from(GRAY_500);
            }
        }

        flash_timer.timer.reset();
        flash_timer.progress = 0.0;
        flash_timer.is_active = true;
    }
}
