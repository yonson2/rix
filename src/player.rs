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

// New event for wall transitions
#[derive(Event)]
pub struct WallTransitionEvent {
    pub from: WallPosition,
    pub to: WallPosition,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WallTransitionEvent>() // Register the event
            .add_systems(Startup, spawn_player)
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
    mut wall_transition_events: EventWriter<WallTransitionEvent>, // Add event writer
) {
    let window = windows.single();
    let (width, height) = (window.width(), window.height());

    let playable_width = width - 2.0 * WALL_MARGIN;
    let playable_height = height - 2.0 * WALL_MARGIN;
    let half_playable_width = playable_width / 2.0;
    let half_playable_height = playable_height / 2.0;

    let left_boundary = -half_playable_width + PLAYER_RADIUS / 2.;
    let right_boundary = half_playable_width - PLAYER_RADIUS / 2.;
    let bottom_boundary = -half_playable_height + PLAYER_RADIUS / 2.;
    let top_boundary = half_playable_height - PLAYER_RADIUS / 2.;

    let bottom_y = -half_playable_height + WALL_THICKNESS - 8.;
    let top_y = half_playable_height - WALL_THICKNESS + 8.;
    let left_x = -half_playable_width + WALL_THICKNESS - 8.;
    let right_x = half_playable_width - WALL_THICKNESS + 8.;

    if let Ok((mut transform, mut wall_position)) = player.get_single_mut() {
        let mut direction = 0.0;
        let current_wall = *wall_position;
        let mut should_change_wall = false;
        let mut new_wall_position = current_wall;

        // Get movement direction based on current wall position
        match current_wall {
            WallPosition::Bottom => {
                if keyboard_input.pressed(KeyCode::ArrowLeft) {
                    direction -= 1.0;

                    // Check if at left edge and should move to left wall
                    if transform.translation.x <= left_boundary + 1.0 {
                        should_change_wall = true;
                        new_wall_position = WallPosition::Left;
                    }
                }
                if keyboard_input.pressed(KeyCode::ArrowRight) {
                    direction += 1.0;

                    // Check if at right edge and should move to right wall
                    if transform.translation.x >= right_boundary - 1.0 {
                        should_change_wall = true;
                        new_wall_position = WallPosition::Right;
                    }
                }

                if should_change_wall {
                    // Emit wall transition event
                    wall_transition_events.send(WallTransitionEvent {
                        from: current_wall,
                        to: new_wall_position,
                    });

                    // Change wall
                    *wall_position = new_wall_position;
                    match new_wall_position {
                        WallPosition::Left => {
                            transform.translation.x = left_x;
                            transform.translation.y = bottom_boundary;
                        }
                        WallPosition::Right => {
                            transform.translation.x = right_x;
                            transform.translation.y = bottom_boundary;
                        }
                        _ => {}
                    }
                } else {
                    // Normal movement on bottom wall
                    let new_x =
                        transform.translation.x + direction * PLAYER_SPEED * time.delta_secs();
                    transform.translation.x = new_x.clamp(left_boundary, right_boundary);
                }
            }
            WallPosition::Right => {
                if keyboard_input.pressed(KeyCode::ArrowUp) {
                    direction += 1.0;

                    // Check if at top edge and should move to top wall
                    if transform.translation.y >= top_boundary - 1.0 {
                        should_change_wall = true;
                        new_wall_position = WallPosition::Top;
                    }
                }
                if keyboard_input.pressed(KeyCode::ArrowDown) {
                    direction -= 1.0;

                    // Check if at bottom edge and should move to bottom wall
                    if transform.translation.y <= bottom_boundary + 1.0 {
                        should_change_wall = true;
                        new_wall_position = WallPosition::Bottom;
                    }
                }

                if should_change_wall {
                    // Emit wall transition event
                    wall_transition_events.send(WallTransitionEvent {
                        from: current_wall,
                        to: new_wall_position,
                    });

                    // Change wall
                    *wall_position = new_wall_position;
                    match new_wall_position {
                        WallPosition::Bottom => {
                            transform.translation.y = bottom_y;
                            transform.translation.x = right_boundary;
                        }
                        WallPosition::Top => {
                            transform.translation.y = top_y;
                            transform.translation.x = right_boundary;
                        }
                        _ => {}
                    }
                } else {
                    // Normal movement on right wall
                    let new_y =
                        transform.translation.y + direction * PLAYER_SPEED * time.delta_secs();
                    transform.translation.y = new_y.clamp(bottom_boundary, top_boundary);
                }
            }
            WallPosition::Top => {
                if keyboard_input.pressed(KeyCode::ArrowLeft) {
                    direction -= 1.0;

                    // Check if at left edge and should move to left wall
                    if transform.translation.x <= left_boundary + 1.0 {
                        should_change_wall = true;
                        new_wall_position = WallPosition::Left;
                    }
                }
                if keyboard_input.pressed(KeyCode::ArrowRight) {
                    direction += 1.0;

                    // Check if at right edge and should move to right wall
                    if transform.translation.x >= right_boundary - 1.0 {
                        should_change_wall = true;
                        new_wall_position = WallPosition::Right;
                    }
                }

                if should_change_wall {
                    // Emit wall transition event
                    wall_transition_events.send(WallTransitionEvent {
                        from: current_wall,
                        to: new_wall_position,
                    });

                    // Change wall
                    *wall_position = new_wall_position;
                    match new_wall_position {
                        WallPosition::Left => {
                            transform.translation.x = left_x;
                            transform.translation.y = top_boundary;
                        }
                        WallPosition::Right => {
                            transform.translation.x = right_x;
                            transform.translation.y = top_boundary;
                        }
                        _ => {}
                    }
                } else {
                    // Normal movement on top wall
                    let new_x =
                        transform.translation.x + direction * PLAYER_SPEED * time.delta_secs();
                    transform.translation.x = new_x.clamp(left_boundary, right_boundary);
                }
            }
            WallPosition::Left => {
                if keyboard_input.pressed(KeyCode::ArrowUp) {
                    direction += 1.0;

                    // Check if at top edge and should move to top wall
                    if transform.translation.y >= top_boundary - 1.0 {
                        should_change_wall = true;
                        new_wall_position = WallPosition::Top;
                    }
                }
                if keyboard_input.pressed(KeyCode::ArrowDown) {
                    direction -= 1.0;

                    // Check if at bottom edge and should move to bottom wall
                    if transform.translation.y <= bottom_boundary + 1.0 {
                        should_change_wall = true;
                        new_wall_position = WallPosition::Bottom;
                    }
                }

                if should_change_wall {
                    // Emit wall transition event
                    wall_transition_events.send(WallTransitionEvent {
                        from: current_wall,
                        to: new_wall_position,
                    });

                    // Change wall
                    *wall_position = new_wall_position;
                    match new_wall_position {
                        WallPosition::Bottom => {
                            transform.translation.y = bottom_y;
                            transform.translation.x = left_boundary;
                        }
                        WallPosition::Top => {
                            transform.translation.y = top_y;
                            transform.translation.x = left_boundary;
                        }
                        _ => {}
                    }
                } else {
                    // Normal movement on left wall
                    let new_y =
                        transform.translation.y + direction * PLAYER_SPEED * time.delta_secs();
                    transform.translation.y = new_y.clamp(bottom_boundary, top_boundary);
                }
            }
        }
    }
}
