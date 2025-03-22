use bevy::{color::palettes::tailwind::CYAN_100, prelude::*};

use crate::wall::{WALL_MARGIN, WALL_THICKNESS};

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

// Player movement configuration
const PLAYER_SPEED: f32 = 300.0;
const PLAYER_RADIUS: f32 = 16.0; // Outer radius of the player

// Wall position tracking
#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallPosition {
    #[default]
    Bottom,
    Right,
    Top,
    Left,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (resize_player, move_player));
    }
}

fn spawn_player(
    mut commands: Commands,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.single();
    let (_, height) = (window.width(), window.height());

    let playable_height = height - 2.0 * WALL_MARGIN;
    let half_playable_height = playable_height / 2.0;

    let player_material = materials.add(Color::from(CYAN_100));

    let player_y_position = -half_playable_height + WALL_THICKNESS - 8.;

    commands.spawn((
        Player,
        WallPosition::default(),
        Mesh2d(meshes.add(Annulus::new(9., 16.))),
        MeshMaterial2d(player_material.clone()),
        Transform::from_xyz(0.0, player_y_position, 2.0),
    ));
}

fn resize_player(
    mut player: Query<(&mut Transform, &WallPosition), With<Player>>,
    windows: Query<&Window, Changed<Window>>,
) {
    // Only proceed if the window has changed
    let Ok(window) = windows.get_single() else {
        return;
    };

    let (width, height) = (window.width(), window.height());

    // Calculate the playable area dimensions (accounting for margins)
    let playable_width = width - 2.0 * WALL_MARGIN;
    let playable_height = height - 2.0 * WALL_MARGIN;
    let half_playable_width = playable_width / 2.0;
    let half_playable_height = playable_height / 2.0;

    if let Ok((mut transform, wall_position)) = player.get_single_mut() {
        match wall_position {
            WallPosition::Bottom => {
                let player_y_position = -half_playable_height + WALL_THICKNESS - 8.;
                transform.translation.y = player_y_position;
                transform.translation.x = transform.translation.x.clamp(
                    -half_playable_width + WALL_THICKNESS + PLAYER_RADIUS,
                    half_playable_width - WALL_THICKNESS - PLAYER_RADIUS,
                );
            }
            WallPosition::Right => {
                let player_x_position = half_playable_width - WALL_THICKNESS + 8.;
                transform.translation.x = player_x_position;
                transform.translation.y = transform.translation.y.clamp(
                    -half_playable_height + WALL_THICKNESS + PLAYER_RADIUS,
                    half_playable_height - WALL_THICKNESS - PLAYER_RADIUS,
                );
            }
            WallPosition::Top => {
                let player_y_position = half_playable_height - WALL_THICKNESS + 8.;
                transform.translation.y = player_y_position;
                transform.translation.x = transform.translation.x.clamp(
                    -half_playable_width + WALL_THICKNESS + PLAYER_RADIUS,
                    half_playable_width - WALL_THICKNESS - PLAYER_RADIUS,
                );
            }
            WallPosition::Left => {
                let player_x_position = -half_playable_width + WALL_THICKNESS - 8.;
                transform.translation.x = player_x_position;
                transform.translation.y = transform.translation.y.clamp(
                    -half_playable_height + WALL_THICKNESS + PLAYER_RADIUS,
                    half_playable_height - WALL_THICKNESS - PLAYER_RADIUS,
                );
            }
        }
    }
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    windows: Query<&Window>,
    mut player: Query<(&mut Transform, &mut WallPosition), With<Player>>,
) {
    let window = windows.single();
    let (width, height) = (window.width(), window.height());

    let playable_width = width - 2.0 * WALL_MARGIN;
    let playable_height = height - 2.0 * WALL_MARGIN;
    let half_playable_width = playable_width / 2.0;
    let half_playable_height = playable_height / 2.0;

    if let Ok((mut transform, mut wall_position)) = player.get_single_mut() {
        let mut direction = 0.0;

        // Get movement direction based on current wall position
        match *wall_position {
            WallPosition::Bottom => {
                if keyboard_input.pressed(KeyCode::ArrowLeft) {
                    direction -= 1.0;
                }
                if keyboard_input.pressed(KeyCode::ArrowRight) {
                    direction += 1.0;
                }

                // Check if player should move to left or right wall
                let new_x = transform.translation.x + direction * PLAYER_SPEED * time.delta_secs();
                let left_boundary = -half_playable_width + WALL_THICKNESS + PLAYER_RADIUS;
                let right_boundary = half_playable_width - WALL_THICKNESS - PLAYER_RADIUS;

                if new_x <= left_boundary && direction < 0.0 {
                    // Move to left wall
                    *wall_position = WallPosition::Left;
                    transform.translation.x = -half_playable_width + WALL_THICKNESS - 8.;
                    transform.translation.y =
                        -half_playable_height + WALL_THICKNESS + PLAYER_RADIUS;
                } else if new_x >= right_boundary && direction > 0.0 {
                    // Move to right wall
                    *wall_position = WallPosition::Right;
                    transform.translation.x = half_playable_width - WALL_THICKNESS + 8.;
                    transform.translation.y =
                        -half_playable_height + WALL_THICKNESS + PLAYER_RADIUS;
                } else {
                    // Stay on bottom wall
                    transform.translation.x = new_x.clamp(left_boundary, right_boundary);
                }
            }
            WallPosition::Right => {
                if keyboard_input.pressed(KeyCode::ArrowUp) {
                    direction += 1.0;
                }
                if keyboard_input.pressed(KeyCode::ArrowDown) {
                    direction -= 1.0;
                }

                // Check if player should move to top or bottom wall
                let new_y = transform.translation.y + direction * PLAYER_SPEED * time.delta_secs();
                let bottom_boundary = -half_playable_height + WALL_THICKNESS + PLAYER_RADIUS;
                let top_boundary = half_playable_height - WALL_THICKNESS - PLAYER_RADIUS;

                if new_y <= bottom_boundary && direction < 0.0 {
                    // Move to bottom wall
                    *wall_position = WallPosition::Bottom;
                    transform.translation.y = -half_playable_height + WALL_THICKNESS - 8.;
                    transform.translation.x = half_playable_width - WALL_THICKNESS - PLAYER_RADIUS;
                } else if new_y >= top_boundary && direction > 0.0 {
                    // Move to top wall
                    *wall_position = WallPosition::Top;
                    transform.translation.y = half_playable_height - WALL_THICKNESS + 8.;
                    transform.translation.x = half_playable_width - WALL_THICKNESS - PLAYER_RADIUS;
                } else {
                    // Stay on right wall
                    transform.translation.y = new_y.clamp(bottom_boundary, top_boundary);
                }
            }
            WallPosition::Top => {
                if keyboard_input.pressed(KeyCode::ArrowLeft) {
                    direction -= 1.0;
                }
                if keyboard_input.pressed(KeyCode::ArrowRight) {
                    direction += 1.0;
                }

                // Check if player should move to left or right wall
                let new_x = transform.translation.x + direction * PLAYER_SPEED * time.delta_secs();
                let left_boundary = -half_playable_width + WALL_THICKNESS + PLAYER_RADIUS;
                let right_boundary = half_playable_width - WALL_THICKNESS - PLAYER_RADIUS;

                if new_x <= left_boundary && direction < 0.0 {
                    // Move to left wall
                    *wall_position = WallPosition::Left;
                    transform.translation.x = -half_playable_width + WALL_THICKNESS - 8.;
                    transform.translation.y = half_playable_height - WALL_THICKNESS - PLAYER_RADIUS;
                } else if new_x >= right_boundary && direction > 0.0 {
                    // Move to right wall
                    *wall_position = WallPosition::Right;
                    transform.translation.x = half_playable_width - WALL_THICKNESS + 8.;
                    transform.translation.y = half_playable_height - WALL_THICKNESS - PLAYER_RADIUS;
                } else {
                    // Stay on top wall
                    transform.translation.x = new_x.clamp(left_boundary, right_boundary);
                }
            }
            WallPosition::Left => {
                if keyboard_input.pressed(KeyCode::ArrowUp) {
                    direction += 1.0;
                }
                if keyboard_input.pressed(KeyCode::ArrowDown) {
                    direction -= 1.0;
                }

                // Check if player should move to top or bottom wall
                let new_y = transform.translation.y + direction * PLAYER_SPEED * time.delta_secs();
                let bottom_boundary = -half_playable_height + WALL_THICKNESS + PLAYER_RADIUS;
                let top_boundary = half_playable_height - WALL_THICKNESS - PLAYER_RADIUS;

                if new_y <= bottom_boundary && direction < 0.0 {
                    // Move to bottom wall
                    *wall_position = WallPosition::Bottom;
                    transform.translation.y = -half_playable_height + WALL_THICKNESS - 8.;
                    transform.translation.x = -half_playable_width + WALL_THICKNESS + PLAYER_RADIUS;
                } else if new_y >= top_boundary && direction > 0.0 {
                    // Move to top wall
                    *wall_position = WallPosition::Top;
                    transform.translation.y = half_playable_height - WALL_THICKNESS + 8.;
                    transform.translation.x = -half_playable_width + WALL_THICKNESS + PLAYER_RADIUS;
                } else {
                    // Stay on left wall
                    transform.translation.y = new_y.clamp(bottom_boundary, top_boundary);
                }
            }
        }
    }
}
