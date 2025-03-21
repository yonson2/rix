use bevy::{color::palettes::basic::PURPLE, prelude::*, window::PrimaryWindow};
use rand::{Rng, thread_rng};

use crate::camera::get_cursor_world_position;
use crate::chain::{ChainHead, ChainNode, move_chain, rotate};
use crate::wall::Border;

const SNAKE_SPEED: f32 = 500.;
const DIRECTION_CHANGE_INTERVAL: f32 = 0.5;
const WALL_DETECTION_DISTANCE: f32 = 200.0;

#[derive(Component)]
pub struct AutonomousMovement {
    pub timer: Timer,
    pub target_direction: Vec2,
    pub current_direction: Vec2,
    pub smoothing_factor: f32,
}

impl Default for AutonomousMovement {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(DIRECTION_CHANGE_INTERVAL, TimerMode::Repeating),
            target_direction: Vec2::new(0.0, 1.0),
            current_direction: Vec2::new(0.0, 1.0),
            smoothing_factor: 0.1,
        }
    }
}

pub struct SnakePlugin;

impl Plugin for SnakePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_snake)
            // .add_systems(Update, move_snakes.before(move_chain))
            .add_systems(Update, move_snake_autonomously.before(move_chain));
    }
}

/// Adds autonomous movement to a snake head entity.
/// This allows the snake to move randomly with wall avoidance.
pub fn enable_autonomous_movement(
    commands: &mut Commands,
    head_entity: Entity,
    smoothing_factor: Option<f32>,
) {
    commands.entity(head_entity).insert(AutonomousMovement {
        smoothing_factor: smoothing_factor.unwrap_or(0.1),
        ..Default::default()
    });
}

fn move_snake_autonomously(
    mut q_heads: Query<(
        &mut Transform,
        &ChainNode,
        &mut ChainHead,
        &mut AutonomousMovement,
    )>,
    q_nodes: Query<(&Transform, &ChainNode), Without<ChainHead>>,
    q_walls: Query<&Transform, (With<Border>, Without<ChainHead>)>,
    time: Res<Time>,
) {
    let mut rng = thread_rng();

    for (mut transform, node, mut head, mut auto_move) in &mut q_heads {
        // Update timer
        auto_move.timer.tick(time.delta());

        // Generate a new random direction if the timer just finished
        if auto_move.timer.just_finished() {
            // Generate random angle in radians
            let angle = rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI);
            auto_move.target_direction = Vec2::new(angle.cos(), angle.sin());
        }

        // Check for nearby walls and adjust direction
        let current_pos = transform.translation.truncate();
        let mut wall_avoidance = Vec2::ZERO;

        for wall_transform in &q_walls {
            let wall_pos = wall_transform.translation.truncate();

            // For simplicity, approximate the wall size based on its position
            // Top/bottom walls are wide and thin, left/right walls are tall and narrow
            let half_size: Vec2;

            // Determine wall dimensions based on position
            if wall_transform.translation.y.abs() > wall_transform.translation.x.abs() {
                // Top or bottom wall (wide and thin)
                half_size = Vec2::new(500.0, 25.0); // Approximate width and half-thickness
            } else {
                // Left or right wall (tall and narrow)
                half_size = Vec2::new(25.0, 400.0); // Approximate half-thickness and height
            }

            // Calculate the closest point on the rectangle to the snake
            let closest_x =
                (wall_pos.x - half_size.x).max(current_pos.x.min(wall_pos.x + half_size.x));
            let closest_y =
                (wall_pos.y - half_size.y).max(current_pos.y.min(wall_pos.y + half_size.y));
            let closest_point = Vec2::new(closest_x, closest_y);

            // Vector from snake to the closest point on the wall
            let to_wall = closest_point - current_pos;
            let distance = to_wall.length();

            // Only consider walls that are within detection range
            if distance < WALL_DETECTION_DISTANCE {
                // Calculate "repulsive force" away from the wall (stronger as we get closer)
                let repulsion_strength = (1.0 - (distance / WALL_DETECTION_DISTANCE)).powf(2.0);
                let repulsion = -to_wall.normalize_or_zero() * repulsion_strength * 2.0;
                wall_avoidance += repulsion;
            }
        }

        // If there's a significant avoidance vector, use it to influence the direction
        if wall_avoidance.length_squared() > 0.01 {
            auto_move.target_direction =
                (auto_move.target_direction + wall_avoidance * 2.0).normalize();
        }

        // Smooth the direction change
        auto_move.current_direction = auto_move
            .current_direction
            .lerp(auto_move.target_direction, auto_move.smoothing_factor)
            .normalize();

        // Apply chain constraints for smooth turning similar to move_snakes
        let mut direction = auto_move.current_direction;
        let next_entity = node.next;
        if let Some(next) = next_entity {
            if let Ok((next_transform, _)) = q_nodes.get(next) {
                let predicted_direction =
                    (transform.translation - next_transform.translation).truncate();
                let angle = predicted_direction
                    .angle_to(direction)
                    .to_degrees()
                    .clamp(-20., 20.);

                direction = rotate(predicted_direction, angle.to_radians());
            }
        }

        // Update head direction and position
        head.direction = direction;
        transform.translation +=
            (direction.normalize_or_zero() * SNAKE_SPEED * time.delta_secs()).extend(0.0);
        transform.rotation = Quat::from_axis_angle(Vec3::Z, direction.to_angle());
    }
}

fn spawn_snake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let snake_size = 20.;
    let length = 30;

    let mut prev_entity = None;

    for i in (0..length).rev() {
        let mut entity = commands.spawn((
            ChainNode {
                next: prev_entity,
                distance: snake_size,
                angle_threshold: (20.0f32).to_radians(),
            },
            Mesh2d(meshes.add(Circle::new(snake_size))),
            MeshMaterial2d(materials.add(Color::from(PURPLE))),
            Transform::from_xyz(0., -i as f32 * snake_size, 0.0 - i as f32),
        ));

        if i == 0 {
            entity
                .insert((ChainHead {
                    target: Vec2::new(0.0, 20.0),
                    direction: Vec2::new(0.0, 1.0),
                },))
                .insert(AutonomousMovement {
                    ..Default::default()
                });
        }

        entity.with_children(|parent| {
            parent.spawn((
                Mesh2d(meshes.add(Circle::new(snake_size + 4.))),
                MeshMaterial2d(materials.add(Color::BLACK)),
                Transform::from_xyz(0., 0., -0.1),
            ));
        });

        prev_entity = Some(entity.id());
    }
}

fn move_snakes(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_heads: Query<(&mut Transform, &ChainNode, &mut ChainHead)>,
    q_nodes: Query<(&Transform, &ChainNode), Without<ChainHead>>,
    time: Res<Time>,
) {
    let Some(target_position) = get_cursor_world_position(&q_windows, &q_camera) else {
        return;
    };

    for (mut transform, node, mut head) in &mut q_heads {
        let mut direction = target_position - transform.translation.truncate();
        if direction.length_squared() <= 100.0 {
            return;
        }

        let next_entity = node.next;
        if let Some(next) = next_entity {
            if let Ok((next_transform, _)) = q_nodes.get(next) {
                let predicted_direction =
                    (transform.translation - next_transform.translation).truncate();
                let angle = predicted_direction
                    .angle_to(direction)
                    .to_degrees()
                    .clamp(-20., 20.);

                direction = rotate(predicted_direction, angle.to_radians());
            }
        }

        head.target = target_position;
        head.direction = direction;
        transform.translation +=
            (direction.normalize_or_zero() * SNAKE_SPEED * time.delta_secs()).extend(0.0);
        transform.rotation = Quat::from_axis_angle(Vec3::Z, direction.to_angle());
    }
}
