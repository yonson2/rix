use bevy::{color::palettes::tailwind::GRAY_600, prelude::*};

pub const WALL_THICKNESS: f32 = 15.;
pub const WALL_MARGIN: f32 = 20.;
pub struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_walls)
            .add_systems(Update, resize_walls);
    }
}

#[derive(Component)]
pub struct Border {
    ///0=top, 1=right, 2=bottom, 3=left
    side: usize,
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
}

fn resize_walls(
    mut borders: Query<(&Border, &mut Transform, &mut Mesh2d)>,
    windows: Query<&Window, Changed<Window>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Only proceed if the window has changed
    let Ok(window) = windows.get_single() else {
        return;
    };

    let (width, height) = (window.width(), window.height());

    // Calculate the playable area dimensions (accounting for margins)
    let playable_width = width - 2.0 * WALL_MARGIN;
    let playable_height = height - 2.0 * WALL_MARGIN;

    // Calculate half dimensions for positioning
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
